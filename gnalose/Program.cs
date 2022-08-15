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
      
            string code = string.Empty;
      
            if(args.Length>0)
                code= File.ReadAllText(args[0]);
            else
            {
                Console.WriteLine("No file supplied, enter file name");
                code = File.ReadAllText(Console.ReadLine());
            }
      
            Interpreter interpreter = new Interpreter(Tokenizer.Tokenize(code));

            interpreter.RunAll(Console.WriteLine, () => int.Parse(Console.ReadLine()));
        }


    }
}