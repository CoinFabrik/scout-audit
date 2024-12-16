namespace run_tests
{
    internal class Program
    {
        static int Main(string[] args)
        {
            List<string>? errors = null;
            if (args.Length == 0)
            {
                var pair = new ConcurrentExclusiveSchedulerPair(TaskScheduler.Default, Environment.ProcessorCount);
                var cases = TestUtils.ListTestCases().ToList();
                errors = TestUtils.RunManyTests(cases, pair.ConcurrentScheduler)?.ToList();
                
            }
            else
            {
                errors = new List<string>();
                foreach (var detector in args)
                    errors.AddRange(TestUtils.RunTests(detector));
                if (errors.Count == 0)
                    errors = null;
            }

            if (errors == null)
                return -1;
            TestUtils.PrintErrors(errors);

            return errors.Count > 0 ? -1 : 0;
        }
    }
}
