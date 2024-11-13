using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

namespace run_tests
{
    internal static class TestUtils
    {
        public static IEnumerable<string> ListTestCases()
        {
            foreach (var blockchain in Directory.EnumerateDirectories("detectors").Select(Path.GetFileName))
            {
                var path = $"detectors/{blockchain}/test-cases";
                if (!Directory.Exists(path))
                    continue;
                foreach (var testCase in Directory.EnumerateDirectories(path).Select(Path.GetFileName))
                {
                    if (testCase == "target")
                        continue;
                    if (testCase.StartsWith('.'))
                        continue;
                    yield return $"{blockchain}/{testCase}";
                }
            }
        }

        public static (string blockchain, string testCase) SplitTestCase(string testCase)
        {
            var split = testCase.Split('/');
            if (split.Length != 2)
                throw new Exception($"Invalid test case string {testCase}");
            return (split[0], split[1]);
        }

        public static IEnumerable<string> RunManyTests(IEnumerable<string> detectors)
        {
            var list = detectors.ToList();

            //Warm up detectors.
            var blockchains = list.Select(x => SplitTestCase(x).blockchain).ToHashSet();
            foreach (var blockchain in blockchains)
            {
                Console.WriteLine($"Building detectors for {blockchain}");
                var (code, _, err) = RunProcess("cargo", new[] { "build", "--release" }, Path.Join(new[] { "detectors", blockchain, "detectors" }));
                if (code != 0)
                {
                    AutoConsoleColor.WriteLine(ConsoleColor.Red, $"Building detectors failed: {err}");
                    return null;
                }
            }

            var ret = new HashSet<string>();

            var tasks = detectors.Select(AsyncRunTests).ToList();
            Task.WhenAll(tasks).ContinueWith(x =>
            {
                lock (ret)
                {
                    foreach (var set in x.Result)
                        ret.UnionWith(set);
                }
                
            }).Wait();

            return ret;
        }

        private static async Task<HashSet<string>> AsyncRunTests(string detector)
        {
            return RunTests(detector).ToHashSet();
        }

        public static IEnumerable<string> RunTests(string detector0)
        {
            var ret = new HashSet<string>();
            var (blockchain, detector) = SplitTestCase(detector0);
            var directory = Path.Join(new[] { "detectors", blockchain, "test-cases", detector });

            AutoConsoleColor.WriteLine(ConsoleColor.Green, $"Performing tests in {directory}");

            if (!Directory.Exists(directory))
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Red, "The specified directory does not exist.");
                return ret;
            }

            foreach (var root in Directory.EnumerateDirectories(directory))
            {
                if (!IsRustProject(root))
                    continue;
                RunTestCase(ret, root, blockchain, detector);
            }
            return ret;
        }

        private static void RunTestCase(HashSet<string> errors, string root, string blockchain, string detector)
        {
            bool failed = !RunUnitTests(root, blockchain);
            failed |= !RunScoutTests(blockchain, detector, root);
            if (failed)
            {
                lock (errors)
                    errors.Add(root);
            }
        }

        private static bool RunUnitTests(string root, string blockchain)
        {
            var args = new List<string>
            {
                "test"
            };
            if (blockchain != "ink")
                args.Add("--all-features");
            var sw = new Stopwatch();
            sw.Start();
            var (code, stdout, _) = RunProcess("cargo", args, root);
            sw.Stop();
            PrintResults(code, stdout, CheckType.UnitTest, root, sw.ElapsedMilliseconds);

            return code == 0;
        }

        private static string? GetShortMessage(string detector, string localDetectors, string root)
        {
            var (code, stdout, _) = RunProcess(
                "cargo",
                new string[]
                {
                    "scout-audit",
                    "--filter",
                    detector,
                    "--metadata",
                    "--local-detectors",
                    localDetectors.ToString(),
                },
                root
            );

            if ((stdout?.Length ?? 0) == 0)
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Red, $"Failed to run integration tests in {root} - Metadata returned empty.");
                return null;
            }

            Debug.Assert(stdout != null);

            var detectorMetadata = JsonConvert.DeserializeObject<JToken>(stdout);
            if ((detectorMetadata?.Type ?? JTokenType.Null) != JTokenType.Object)
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Red, $"Failed to extract JSON from metadata.");
                return null;
            }

            Debug.Assert(detectorMetadata != null);

            var value = ((JObject)detectorMetadata)[detector.Replace('-', '_')];
            if ((value?.Type ?? JTokenType.Null) != JTokenType.String)
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Red, $"Failed to extract message from JSON.");
                return null;
            }

            Debug.Assert(value != null);

            return value.ToString();
        }

        private static bool RunScoutTests(string blockchain, string detector, string root)
        {
            var sw = new Stopwatch();
            sw.Start();

            var localDetectors = Path.Join(new[]
                { Environment.CurrentDirectory, "detectors", blockchain, "detectors" });

            if (GetShortMessage(detector, localDetectors, root) == null)
                return false;

            var temp = Path.Join(Path.GetTempPath(), Path.GetTempFileName());

            var (code, _, err) = RunProcess(
                "cargo",
                new []
                {
                    "scout-audit",
                    "--local-detectors",
                    localDetectors,
                    "--output-format",
                    "raw-json",
                    "--output-path",
                    temp,
                },
                root
            );

            if (code != 0)
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Red, "Scout failed to run.");
                return false;
            }

            var shouldFail = root.Contains("vulnerable");
            var didFail = false;

            {
                using var file = new StreamReader(temp);
                var detectorsTriggered = file
                    .ReadToEnd()
                    .Replace("\r\n", "\n")
                    .Replace("\r", "\n")
                    .Split('\n')
                    .Select(JsonConvert.DeserializeObject<ScoutOutputObject>)
                    .Where(x => x != null)
                    .Select(x => x.code.code.Replace('_', '-'))
                    .ToHashSet();
                didFail = detectorsTriggered.Contains(detector);
            }
            File.Delete(temp);

            if (shouldFail != didFail)
            {
                var explanation = didFail ? "it failed when it shouldn't have" : "it didn't fail when it should have";
                AutoConsoleColor.WriteLine(ConsoleColor.Red, $"Test case {root} didn't pass because {explanation}.");
                return false;
            }

            sw.Stop();

            PrintResults(code, err, CheckType.ScoutTest, root, sw.ElapsedMilliseconds);

            return true;
        }

        private enum CheckType
        {
            Clippy,
            Format,
            Udeps,
            UnitTest,
            ScoutTest,
        }

        private static void PrintResults(int code, string message, CheckType type, string root, long milliseconds)
        {
            string issueType;
            string actionType;
            switch (type)
            {
                case CheckType.Clippy:
                case CheckType.Format:
                case CheckType.Udeps:
                    issueType = "issues";
                    actionType = "check";
                    break;
                case CheckType.UnitTest:
                case CheckType.ScoutTest:
                    issueType = "errors";
                    actionType = "run";
                    break;
                default:
                    throw new ArgumentOutOfRangeException($"Check type == {type}");
            }

            using var ac = new AutoConsoleColor(code == 0 ? ConsoleColor.Green : ConsoleColor.Red);
            Console.WriteLine($"[{milliseconds * 0.001:F2}] - Completed {type} {actionType} in: {root}");
            if (code != 0)
            {
                Console.WriteLine($"{type} {issueType} found in: {root}");
                if (message.Length > 0)
                {
                    message = message.Replace("\r\n", "\n").Replace("\r", "\n");
                    foreach (var line in message.Split('\n'))
                        Console.WriteLine($"| {line}");
                    Console.WriteLine();
                }
            }
        }

        public static (int returnCode, string stdout, string stderr) RunProcess(string command,
            IEnumerable<string> arguments, string workingDirectory)
        {
            var psi = new ProcessStartInfo("cargo", arguments)
            {
                WorkingDirectory = workingDirectory,
                CreateNoWindow = true,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
            };

            using var process = new Process()
            {
                StartInfo = psi
            };

            // Capture output asynchronously to avoid buffer-related deadlocks
            var @out = new StringBuilder();
            var err = new StringBuilder();
            //var outputTask = new TaskCompletionSource<string>();
            //var errorTask = new TaskCompletionSource<string>();

            process.OutputDataReceived += (sender, e) =>
            {
                if (e.Data != null)
                    @out.Append(e.Data + Environment.NewLine);
                //if (e.Data == null)
                //    outputTask.TrySetResult(string.Empty); // End of stream
                //else
                //    outputTask.TrySetResult(e.Data);
            };

            process.ErrorDataReceived += (sender, e) =>
            {
                if (e.Data != null)
                    err.Append(e.Data + Environment.NewLine);
                //if (e.Data == null)
                //    errorTask.TrySetResult(string.Empty); // End of stream
                //else
                //    errorTask.TrySetResult(e.Data);
            };

            process.Start();
            process.BeginOutputReadLine();
            process.BeginErrorReadLine();
            process.WaitForExit();
            //outputTask.Task.Wait();
            //errorTask.Task.Wait();
            //
            //var stdout = outputTask.Task.Result;
            //var stderr = errorTask.Task.Result;

            //return (process.ExitCode, stdout, stderr);
            return (process.ExitCode, @out.ToString(), err.ToString());
        }

        private static bool IsRustProject(string root)
        {
            return File.Exists(Path.Join(root, "Cargo.toml")) &&
                   File.Exists(Path.Join(new[] { root, "src", "lib.rs" }));
        }

        public static void PrintErrors(IEnumerable<string> errors)
        {
            var errorList = errors.ToList();
            if (errorList.Count > 0)
            {
                var ac = new AutoConsoleColor(ConsoleColor.Red);
                Console.WriteLine("Errors detected in the following directories:");
                foreach (var error in errorList)
                    Console.WriteLine($"• {error}");
            }
            else
            {
                AutoConsoleColor.WriteLine(ConsoleColor.Green, "No errors.");
            }
        }
    }
}
