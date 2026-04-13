using System;
using System.Linq;
using Newtonsoft.Json.Linq;
using NUnit.Framework;
using UnityCliBridge.Core;

namespace UnityCliBridge.Tests.Editor.Core
{
    [TestFixture]
    public class BridgeCommandStatsTests
    {
        [SetUp]
        public void SetUp()
        {
            BridgeCommandStats.ResetForTesting();
        }

        [TearDown]
        public void TearDown()
        {
            BridgeCommandStats.ResetForTesting();
        }

        [Test]
        public void CaptureSnapshot_AggregatesPerCommandTimingsAndStages()
        {
            var enqueuedAt = DateTime.UtcNow.AddMilliseconds(-25);

            using (var session = BridgeCommandStats.BeginCommand("capture_screenshot", enqueuedAt))
            {
                BridgeCommandStats.RecordStageDuration("handler_ms", 10.25);
                BridgeCommandStats.RecordStageDuration("encode_ms", 4.75);
                BridgeCommandStats.RecordStageDuration("io_ms", 2.5);
                session.Complete(true, 128);
            }

            using (var session = BridgeCommandStats.BeginCommand("capture_screenshot", enqueuedAt))
            {
                BridgeCommandStats.RecordStageDuration("handler_ms", 6.5);
                BridgeCommandStats.RecordStageDuration("encode_ms", 2.25);
                session.Complete(true, 64);
            }

            JObject snapshot = JObject.FromObject(BridgeCommandStats.CaptureSnapshotForTesting());

            Assert.AreEqual(2, snapshot["counts"]?["capture_screenshot"]?.Value<int>());
            Assert.AreEqual("capture_screenshot", snapshot["recent"]?[0]?["type"]?.Value<string>());

            var command = snapshot["timings"]?["capture_screenshot"];
            Assert.IsNotNull(command);
            Assert.AreEqual(2, command["count"]?.Value<int>());
            Assert.Greater(command["totalMs"]?.Value<double>() ?? 0d, 0d);
            Assert.Greater(command["avgMs"]?.Value<double>() ?? 0d, 0d);

            var handler = command["stages"]?["handler_ms"];
            Assert.IsNotNull(handler);
            Assert.AreEqual(2, handler["count"]?.Value<int>());
            Assert.AreEqual(16.75d, handler["totalMs"]?.Value<double>());

            var queue = command["stages"]?["queue_ms"];
            Assert.IsNotNull(queue);
            Assert.AreEqual(2, queue["count"]?.Value<int>());
            Assert.Greater(queue["lastMs"]?.Value<double>() ?? 0d, 0d);

            var io = command["stages"]?["io_ms"];
            Assert.IsNotNull(io);
            Assert.AreEqual(1, io["count"]?.Value<int>());
            Assert.AreEqual(2.5d, io["lastMs"]?.Value<double>());
        }

        [Test]
        public void RecordStageDuration_WithoutActiveCommand_IsIgnored()
        {
            BridgeCommandStats.RecordStageDuration("encode_ms", 5.0);

            JObject snapshot = JObject.FromObject(BridgeCommandStats.CaptureSnapshotForTesting());

            Assert.AreEqual(0, snapshot["counts"]?.Count());
            Assert.AreEqual(0, snapshot["timings"]?.Count());
        }

        [Test]
        public void CommandContext_IsExposedToFriendTestAssembly()
        {
            Assert.IsTrue(typeof(BridgeCommandStats.CommandContext).IsNestedAssembly);
        }
    }
}
