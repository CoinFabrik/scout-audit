using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace run_tests
{
    internal static class Utility
    {
        public static T? FirstOrNull<T>(this IEnumerable<T> xs) where T : class
        {
            foreach (var x in xs)
                return x;
            return null;
        }

        public static void Shuffle<T>(this List<T> xs, Random? rng = null)
        {
            rng ??= new Random();
            for (int i = 0; i < xs.Count - 1; i++)
            {
                var j = rng.Next(i, xs.Count);
                if (i == j)
                    continue;
                (xs[i], xs[j]) = (xs[j], xs[i]);
            }
        }
    }
}
