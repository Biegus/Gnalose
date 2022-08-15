using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;

namespace Gnalose
{
    public enum OpCode
    {
        UNKNOWN,
        OP_DEF,
        OP_DEF_A,
        OP_UNDEF,
        OP_READ,
        OP_PRINT,
        OP_ADD,
        OP_SUB,
        OP_END_IF,
        OP_GOTO,
        OP_REMB,
        OP_MARK,
        OP_PRINT_ASCI,
        OP_UNDF_AR,
        OP_MULT,
        OP_DIV,
        OP_UNMARK,
        OP_IF_LE=30,
        OP_IF_E=31,
        OP_IF_GE=32,
        OP_IF_NE=33,
        OP_IF_G=34,
        OP_IF_L=35,
     
    }

    public struct Reference
    {
        private static Regex indexRegex = new Regex(@"(?<name>\w+?)\[(?<index>\w+?)\]");
        public string Name;
        public RefIndex? Index;
        public static implicit operator Reference(string txt)
        {
            if (txt.Contains("["))
            {
                var res = indexRegex.Match(txt);
                return new Reference() {Name = res.Groups["name"].Value, Index =  RefIndex.Make( res.Groups["index"].Value)};
            }
            return new Reference() {Name = txt};
        }
    }

    public struct RefIndex
    {
        public bool RefMode { get; private set; }
        public string Ref { get; private set; }
        public int Literal { get; private set; }

        public static RefIndex Make(string txt)
        {
            if (int.TryParse(txt, out int res))
                return new RefIndex  {RefMode = false, Literal = res};
            else return new RefIndex  {RefMode = true, Ref = txt};
        }
        
    }
    public struct UnionRef
    {
        public bool RefMode { get; private set; }
        public Reference Reference { get; private set; }
        public int Literal { get; private set; }

        public static UnionRef Make(string txt)
        {
            if (int.TryParse(txt, out int res))
                return new UnionRef() {RefMode = false, Literal = res};
            else return new UnionRef() {RefMode = true, Reference = txt};
        }

        public static UnionRef FromReference(string rf)
        {
            return new UnionRef() {RefMode = true, Reference =rf};
        }
    }
    public struct Token
    {
        public OpCode OpCode;
        public UnionRef A;
        public UnionRef B;
        public Token(OpCode opCode, UnionRef a, UnionRef b)
        {
            OpCode = opCode;
            A = a;
            B = b;
        }
    }

    public struct IfPaths
    {
        public IReadOnlyDictionary<int, int> Dict { get; }

        public IfPaths(IReadOnlyDictionary<int,int> dict)
        {
            Dict = dict;
        }
    }

    public struct PreProcessed
    {
        public IfPaths Paths { get; }
        public IReadOnlyList<Token> Tokens { get; }

        public PreProcessed(IfPaths paths, IReadOnlyList<Token> tokens)
        {
            Paths = paths;
            Tokens = tokens;
        }
    }

    public class Tokenizer
    {
        public static string StripOffCommands(string line)
        {
            if (!line.Contains("/"))
                return line;
            return line.Split('/')[1];
        }

       
        public static IfPaths BuildPaths(IReadOnlyList<Token> tokens)
        {
            Dictionary<int, int> dict = new();
            Stack<int> stack = new();
            for (int i = 0; i < tokens.Count; i++)
            {
                var token = tokens[i];
                if ((int) token.OpCode > 30)
                {
                    stack.Push(i);
                }
                else if (token.OpCode == OpCode.OP_END_IF)
                {
                    dict[stack.Pop()] = i;
                }
            }

            return new IfPaths(dict);
        }

        //code should not be reversed
        public static IEnumerable<string> PreTokenize(string code)
        {
            Stack<string> ifRules = new();
            string[] lines = code.Split('\n');
            string[] linesFixed = new string[lines.Length];
            for (int index = 0; index < lines.Length; index++)
            {
                string line = lines[index];
                line = line.TrimStart();
                if (line.StartsWith("if "))
                {
                    ifRules.Push(line);
                    linesFixed[index] = "fi";
                }

                else if (line.StartsWith("fi"))
                {
                    string poped = ifRules.Pop();
                    linesFixed[index] = poped;
                }
                else linesFixed[index] = line;
            }

            return linesFixed.Reverse();

        }
        public static PreProcessed Tokenize(string code)
        {
            var tokens = PreTokenize(code).Select(TokenizeSingle)
                .Where(item => item.HasValue).Select(item => item.Value)
                .ToList();
            var paths = BuildPaths(tokens);
            return new PreProcessed(paths, tokens);
        }

        public static Token? TokenizeSingle(string line)
        {
            const string NOT_ENOUGHT_WORDS_TEXT = "{0} requires {1} word";

            line = line.Trim();
         
            line = StripOffCommands(line);
            if (line == string.Empty)
                return default;
            string[] words = line.Split(' ');

            Exception ThrowIndexError(OpCode opCode, int indx)
            {
                throw new InvalidOperationException(string.Format(NOT_ENOUGHT_WORDS_TEXT, opCode, indx));
            }

            string Get(int index, OpCode code = OpCode.UNKNOWN)
            {
                if (words.Length <= index && code == OpCode.UNKNOWN)
                    return string.Empty;
                if (words.Length <= index)
                    ThrowIndexError(code, index);
                else
                    return words[index];
                return string.Empty;
            }

            Token BuildToken(OpCode opcode, int indexA, int indexB = 0)
            {
                return new Token(opcode, UnionRef.Make(Get(indexA, opcode)), UnionRef.Make(Get(indexB, opcode)));
            }
            
            if (Get(0) == "undefine" && Get(1)== "single")
                return BuildToken(OpCode.OP_DEF_A, 2);
            if (Get(0) == "undefine")
                return BuildToken(OpCode.OP_DEF, 1);
            if (Get(0) == "define" && Get(1) == "single")
                return BuildToken(OpCode.OP_UNDF_AR, 2);
            if (Get(0) == "define")
                return BuildToken(OpCode.OP_UNDEF, 1);
            if (Get(0) == "read" && Get(1) == "to")
                return BuildToken(OpCode.OP_PRINT, 2);
            if (Get(0) == "sub" && Get(2) == "from")
                return BuildToken(OpCode.OP_ADD, 1, 3);
            if (Get(0) == "add" && Get(2) == "to")
                return BuildToken(OpCode.OP_SUB, 1, 3);
            if (Get(0) == "read" && Get(1) == "as" && Get(2) == "number" && Get(3) == "to")
                return BuildToken(OpCode.OP_PRINT_ASCI, 4);
            if (Get(0) == "print")
                return BuildToken(OpCode.OP_READ, 1);
            if (Get(0) == "if" && Get(2) == "not" && Get(3) == "equal" && Get(4) == "to")
                return BuildToken(OpCode.OP_IF_E, 1, 5);
            if (Get(0) == "if" && Get(2) == "equal" && Get(3) == "to")
                return BuildToken(OpCode.OP_IF_NE, 1, 4);
            if (Get(0) == "if" && Get(2) == "greater" && Get(3) == "than")
                return BuildToken(OpCode.OP_IF_LE, 1, 4);
            if (Get(0) == "if" && Get(2) == "lower" && Get(3) == "than")
                return BuildToken(OpCode.OP_IF_GE, 1, 4);
            if (Get(0) == "if" && Get(2) == "greater" 
                               && Get(3) == "or" && Get(4)=="equal" 
                               && Get(5)=="than")
                return BuildToken(OpCode.OP_IF_L, 1, 6);
            if (Get(0) == "if" && Get(2) == "lower" && Get(3) == "or" && Get(4)=="equal" && Get(5)=="than")
                return BuildToken(OpCode.OP_IF_G, 1, 6);
            if (Get(0) == "fi")
                return BuildToken(OpCode.OP_END_IF,0,0);
            if (Get(0) == "read" && Get(1) == "as" && Get(2) == "number" && Get(3) == "to")
                return BuildToken(OpCode.OP_PRINT_ASCI, 4);
            if (Get(0) == "unmark")
                return BuildToken(OpCode.OP_MARK, 1);
            if (Get(0) == "forget")
                return BuildToken(OpCode.OP_REMB, 1);
            if (Get(0) == "halt")
                return BuildToken(OpCode.OP_GOTO, 0);
            if (Get(0) == "div" && Get(2) == "by")
                return BuildToken(OpCode.OP_MULT, 1, 3);
            if (Get(0) == "mult" && Get(2) == "times")
                return BuildToken(OpCode.OP_DIV, 1, 3);
            if (Get(0) == "mark" )
                return BuildToken(OpCode.OP_UNMARK, 1);

            throw new InvalidOperationException($"Unknown command {line}");
            return default;
        }
    }
}