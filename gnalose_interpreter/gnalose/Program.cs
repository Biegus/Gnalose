using System;
using System.IO;
using System.Linq;
using System.Text;

namespace Gnalose
{
    internal class Program
    {
        public static void Main(string[] args)
        {
            
            string fileName;
            if (args.Length > 0)
                fileName = args[0];
            else
            {
                Console.WriteLine("No file supplied, enter file name");
                fileName=Console.ReadLine();
            }
            if (!File.Exists(fileName))
            {
                Console.WriteLine("File doesn't exist");
                return;
            }
            
            string code;
            try
            {
                 code = File.ReadAllText(fileName);
            }
            catch (Exception exc)
            {
                Console.WriteLine($"Error while opening file {exc.Message}");
                return;
            }
            try
            {
                Interpreter interpreter = new(Tokenizer.Tokenize(code));
                interpreter.RunAll(Console.WriteLine, () => int.Parse(Console.ReadLine()));
            }
            catch (GnaloseException exc)
            {
                Console.WriteLine("-----");
                Console.WriteLine($"{exc.Message}");
            }
            catch (Exception exc)
            {
                Console.WriteLine($"Inner fatal error:{exc}");
            }
           
        }


    }
}