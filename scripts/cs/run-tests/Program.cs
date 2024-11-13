namespace run_tests
{
    internal class Program
    {
        static int Main(string[] args)
        {
            var cases = TestUtils.ListTestCases().ToList();
            var errors = TestUtils.RunManyTests(cases)?.ToList();
            if (errors == null)
                return -1;
            TestUtils.PrintErrors(errors);

            return errors.Count > 0 ? -1 : 0;
        }
    }
}
