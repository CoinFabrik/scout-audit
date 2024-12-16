using System.IO;

namespace run_tests;

class AutoConsoleColor : IDisposable
{
    private ConsoleColor? _old;
    private static Mutex _lock = new();

    public AutoConsoleColor(ConsoleColor c)
    {
        _lock.WaitOne();
        _old = Console.ForegroundColor;
        Console.ForegroundColor = c;
    }

    public void Dispose()
    {
        if (_old != null)
        {
            Console.ForegroundColor = _old.Value;
            _old = null;
            _lock.ReleaseMutex();
        }
    }

    public static void WriteLine(ConsoleColor c, string message)
    {
        using var cc = new AutoConsoleColor(c);
        Console.WriteLine(message);
    }
}
