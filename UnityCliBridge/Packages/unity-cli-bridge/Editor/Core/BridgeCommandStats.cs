using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading;

namespace UnityCliBridge.Core
{
    /// <summary>
    /// Aggregates bridge-side command timing and stage breakdowns for get_command_stats.
    /// </summary>
    public static class BridgeCommandStats
    {
        internal sealed class MetricAggregate
        {
            public int Count;
            public double TotalMs;
            public double MaxMs;
            public double LastMs;

            public void Add(double durationMs)
            {
                if (durationMs < 0)
                {
                    durationMs = 0;
                }

                Count++;
                TotalMs += durationMs;
                LastMs = durationMs;
                if (durationMs > MaxMs)
                {
                    MaxMs = durationMs;
                }
            }
        }

        private sealed class CommandAggregate
        {
            public int Count;
            public int ErrorCount;
            public int LastResponseBytes;
            public double TotalMs;
            public double MaxMs;
            public double LastMs;
            public DateTime LastStartedAtUtc;
            public DateTime LastCompletedAtUtc;
            public readonly Dictionary<string, MetricAggregate> Stages =
                new Dictionary<string, MetricAggregate>(StringComparer.OrdinalIgnoreCase);
        }

        internal sealed class CommandContext
        {
            public CommandContext(string commandType, DateTime startedAtUtc)
            {
                CommandType = string.IsNullOrWhiteSpace(commandType) ? "(null)" : commandType;
                StartedAtUtc = startedAtUtc;
                Stopwatch = Stopwatch.StartNew();
            }

            public string CommandType { get; }
            public DateTime StartedAtUtc { get; }
            public Stopwatch Stopwatch { get; }
            public readonly Dictionary<string, MetricAggregate> Stages =
                new Dictionary<string, MetricAggregate>(StringComparer.OrdinalIgnoreCase);
            public bool Completed { get; set; }
        }

        public sealed class CommandSession : IDisposable
        {
            private readonly CommandContext context;

            internal CommandSession(CommandContext context)
            {
                this.context = context;
            }

            public void Complete(bool success, int responseBytes)
            {
                BridgeCommandStats.Complete(context, success, responseBytes);
            }

            public void Dispose()
            {
                BridgeCommandStats.Complete(context, success: false, responseBytes: 0);
            }
        }

        private static readonly object statsLock = new object();
        private static readonly Dictionary<string, int> commandCounts =
            new Dictionary<string, int>(StringComparer.OrdinalIgnoreCase);
        private static readonly Queue<(DateTime t, string type)> recentCommands =
            new Queue<(DateTime, string)>();
        private static readonly Dictionary<string, CommandAggregate> aggregates =
            new Dictionary<string, CommandAggregate>(StringComparer.OrdinalIgnoreCase);
        private static readonly AsyncLocal<CommandContext> currentContext = new AsyncLocal<CommandContext>();

        public static CommandSession BeginCommand(string commandType, DateTime? queuedAtUtc = null)
        {
            var startedAtUtc = DateTime.UtcNow;
            var normalizedType = string.IsNullOrWhiteSpace(commandType) ? "(null)" : commandType;
            var context = new CommandContext(normalizedType, startedAtUtc);

            lock (statsLock)
            {
                if (!commandCounts.ContainsKey(normalizedType))
                {
                    commandCounts[normalizedType] = 0;
                }

                commandCounts[normalizedType]++;
                recentCommands.Enqueue((startedAtUtc, normalizedType));
                while (recentCommands.Count > 50)
                {
                    recentCommands.Dequeue();
                }
            }

            currentContext.Value = context;

            if (queuedAtUtc.HasValue)
            {
                var queueMs = (startedAtUtc - queuedAtUtc.Value).TotalMilliseconds;
                RecordStageDuration("queue_ms", queueMs);
            }

            return new CommandSession(context);
        }

        public static void RecordStageDuration(string stageName, double durationMs)
        {
            if (string.IsNullOrWhiteSpace(stageName))
            {
                return;
            }

            var context = currentContext.Value;
            if (context == null || context.Completed)
            {
                return;
            }

            if (!context.Stages.TryGetValue(stageName, out var aggregate))
            {
                aggregate = new MetricAggregate();
                context.Stages[stageName] = aggregate;
            }

            aggregate.Add(durationMs);
        }

        public static T MeasureStage<T>(string stageName, Func<T> action)
        {
            var stopwatch = Stopwatch.StartNew();
            try
            {
                return action();
            }
            finally
            {
                stopwatch.Stop();
                RecordStageDuration(stageName, stopwatch.Elapsed.TotalMilliseconds);
            }
        }

        public static void MeasureStage(string stageName, Action action)
        {
            var stopwatch = Stopwatch.StartNew();
            try
            {
                action();
            }
            finally
            {
                stopwatch.Stop();
                RecordStageDuration(stageName, stopwatch.Elapsed.TotalMilliseconds);
            }
        }

        public static object CaptureSnapshot()
        {
            lock (statsLock)
            {
                return new
                {
                    counts = commandCounts.ToDictionary(kv => kv.Key, kv => kv.Value),
                    recent = recentCommands
                        .ToArray()
                        .Select(x => new { timestamp = x.t.ToString("o"), type = x.type })
                        .ToArray(),
                    timings = aggregates.ToDictionary(
                        kv => kv.Key,
                        kv => ToSnapshot(kv.Value))
                };
            }
        }

        public static object CaptureSnapshotForTesting()
        {
            return CaptureSnapshot();
        }

        public static void ResetForTesting()
        {
            lock (statsLock)
            {
                commandCounts.Clear();
                recentCommands.Clear();
                aggregates.Clear();
            }

            currentContext.Value = null;
        }

        private static object ToSnapshot(CommandAggregate aggregate)
        {
            return new
            {
                count = aggregate.Count,
                errorCount = aggregate.ErrorCount,
                totalMs = RoundMs(aggregate.TotalMs),
                avgMs = aggregate.Count > 0 ? RoundMs(aggregate.TotalMs / aggregate.Count) : 0d,
                maxMs = RoundMs(aggregate.MaxMs),
                lastMs = RoundMs(aggregate.LastMs),
                lastResponseBytes = aggregate.LastResponseBytes,
                lastStartedAt = aggregate.LastStartedAtUtc == default ? null : aggregate.LastStartedAtUtc.ToString("o"),
                lastCompletedAt = aggregate.LastCompletedAtUtc == default ? null : aggregate.LastCompletedAtUtc.ToString("o"),
                stages = aggregate.Stages.ToDictionary(
                    stage => stage.Key,
                    stage => new
                    {
                        count = stage.Value.Count,
                        totalMs = RoundMs(stage.Value.TotalMs),
                        avgMs = stage.Value.Count > 0 ? RoundMs(stage.Value.TotalMs / stage.Value.Count) : 0d,
                        maxMs = RoundMs(stage.Value.MaxMs),
                        lastMs = RoundMs(stage.Value.LastMs)
                    })
            };
        }

        private static void Complete(CommandContext context, bool success, int responseBytes)
        {
            if (context == null || context.Completed)
            {
                return;
            }

            context.Completed = true;
            context.Stopwatch.Stop();

            lock (statsLock)
            {
                if (!aggregates.TryGetValue(context.CommandType, out var aggregate))
                {
                    aggregate = new CommandAggregate();
                    aggregates[context.CommandType] = aggregate;
                }

                aggregate.Count++;
                if (!success)
                {
                    aggregate.ErrorCount++;
                }

                var totalMs = context.Stopwatch.Elapsed.TotalMilliseconds;
                aggregate.TotalMs += totalMs;
                aggregate.LastMs = totalMs;
                aggregate.LastStartedAtUtc = context.StartedAtUtc;
                aggregate.LastCompletedAtUtc = DateTime.UtcNow;
                aggregate.LastResponseBytes = Math.Max(0, responseBytes);
                if (totalMs > aggregate.MaxMs)
                {
                    aggregate.MaxMs = totalMs;
                }

                foreach (var stage in context.Stages)
                {
                    if (!aggregate.Stages.TryGetValue(stage.Key, out var target))
                    {
                        target = new MetricAggregate();
                        aggregate.Stages[stage.Key] = target;
                    }

                    target.Count += stage.Value.Count;
                    target.TotalMs += stage.Value.TotalMs;
                    target.LastMs = stage.Value.LastMs;
                    if (stage.Value.MaxMs > target.MaxMs)
                    {
                        target.MaxMs = stage.Value.MaxMs;
                    }
                }
            }

            if (ReferenceEquals(currentContext.Value, context))
            {
                currentContext.Value = null;
            }
        }

        private static double RoundMs(double value)
        {
            return Math.Round(value, 3, MidpointRounding.AwayFromZero);
        }
    }
}
