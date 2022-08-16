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
        public bool RefMode;
        public string Ref;
        public int Literal;

        public static RefIndex Make(string txt)
        {
            if (int.TryParse(txt, out int res))
                return new RefIndex  {RefMode = false, Literal = res};
            else return new RefIndex  {RefMode = true, Ref = txt};
        }
        
    }
    public struct UnionRef
    {
        public bool RefMode;
        public Reference Reference;
        public int Literal;

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
        public int OriginalLineNumber;
        public string OriginalLine;
        public Token(OpCode opCode, UnionRef a, UnionRef b,int orgLineNumber,string originalLine)
        {
            OpCode = opCode;
            A = a;
            B = b;
            OriginalLineNumber = orgLineNumber;
            OriginalLine = originalLine;

        }
    }

    public struct JumpTable
    {
        public IReadOnlyDictionary<int, int> Dict;

        public JumpTable(IReadOnlyDictionary<int,int> dict)
        {
            Dict = dict;
        }
    }

    public struct TokenCollection
    {
        public JumpTable Paths { get; }
        public IReadOnlyList<Token> Tokens { get; }
        public int OriginalLineCount { get; }

        public TokenCollection(JumpTable paths, IReadOnlyList<Token> tokens,int originalLineCount)
        {
            Paths = paths;
            Tokens = tokens;
            OriginalLineCount = originalLineCount;
        }
    }

    public enum TokenPhase
    {
        PreTokenize,
        Tokenize,
        BuildJumpTable
    }
    public class Tokenizer
    {
     
        [Flags]
        private enum TokenBuilderFlags
        {
            NONE=0,
            A_CANNOT_BE_LITERAL=1<<0,
            B_CANNOT_BE_LITERAL=1<<1,
            
            A_OR_B_CANNOT_BE_LITERAL=A_CANNOT_BE_LITERAL+B_CANNOT_BE_LITERAL
        }
        public static string StripOffCommands(string line)
        {
            if (!line.Contains("/"))
                return line;
            return line.Split('/')[1];
        }

       
        public static JumpTable BuildPaths(IReadOnlyList<Token> tokens)
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

            return new JumpTable(dict);
        }

        //code should not be reversed
        public static string[] PreTokenize(string code)
        {
            Stack< (string line,int index)> stack = new();
            string[] lines = code.Split('\n');
            string[] linesFixed = new string[lines.Length];
            for (int index = 0; index < lines.Length; index++)
            {
                string line = lines[index];
                line = line.TrimStart();
                if (line.StartsWith("if "))
                {
                    stack.Push((line,index));
                    linesFixed[index] = "fi";
                }

                else if (line.StartsWith("fi"))
                {
                    if (stack.Count == 0)
                    {
                        throw new GnaloseTokenizerException($"There's no if connected to that fi", index + 1,
                            lines.Length - index, TokenPhase.PreTokenize, line);
                    }
                    string poped = stack.Pop().line;
                    linesFixed[index] = poped;
                }
                else linesFixed[index] = line;
            }

            if (stack.Count != 0)
            {
                var poped = stack.Pop();
                throw new GnaloseTokenizerException($"There's no fi connected to that if",poped.index+1 ,
                    lines.Length-poped.index, TokenPhase.PreTokenize, poped.line);
            }
            return linesFixed;

        }
        public static TokenCollection Tokenize(string code)
        {
            var preTokenized = PreTokenize(code);
            
            var tokens = preTokenized.Reverse().Select((line,i)=>TokenizeSingle(line,preTokenized.Length-i,i+1))
                .Where(item => item.HasValue).Select(item => item.Value)
                .ToList();
            var paths = BuildPaths(tokens);
            return new TokenCollection(paths, tokens,preTokenized.Length);
        }

        public static Token? TokenizeSingle(string line,int lineNumber,int lineBackwards)
        {
            const string NOT_ENOUGHT_WORDS_TEXT = "more than {0} subowrds required";

            
            line = line.Trim();
            
         
            line = StripOffCommands(line);
            if (line == string.Empty)
                return default;
            string[] words = line.Split(' ');

            void ThrowTokenException(string message)
            {
                throw new GnaloseTokenizerException(message,lineNumber, lineBackwards
                    ,TokenPhase.Tokenize,line);
            }
            void ThrowIndexError(OpCode opCode, int indx)
            {
                ThrowTokenException(string.Format(NOT_ENOUGHT_WORDS_TEXT, indx));
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

         
            Token BuildToken(OpCode opcode, int indexA, int indexB =0,TokenBuilderFlags flags= TokenBuilderFlags.NONE)
            {
                const string CANNOT_BE_LITERAL_ERROR = "{0}: argument {1} cannot be literal value";
                UnionRef a = UnionRef.Make(Get(indexA, opcode));
                UnionRef b = UnionRef.Make(Get(indexB, opcode));
                
                if(!a.RefMode && flags.HasFlag( TokenBuilderFlags.A_CANNOT_BE_LITERAL))
                {
                    ThrowTokenException(string.Format(CANNOT_BE_LITERAL_ERROR, opcode, "a"));
                }
                if(!b.RefMode && flags.HasFlag( TokenBuilderFlags.B_CANNOT_BE_LITERAL))
                {
                    ThrowTokenException(string.Format(CANNOT_BE_LITERAL_ERROR, opcode, "b"));
                }

                return new Token(opcode, a, b, lineNumber, line);
            }
            if (Get(0) == "undefine" && Get(1)== "single")
                return BuildToken(OpCode.OP_DEF_A, 2,flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
            if (Get(0) == "undefine")
                return BuildToken(OpCode.OP_DEF, 1,flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
            if (Get(0) == "define" && Get(1) == "single")
                return BuildToken(OpCode.OP_UNDF_AR, 2,flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
            if (Get(0) == "define")
                return BuildToken(OpCode.OP_UNDEF, 1,flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
            if (Get(0) == "read" && Get(1) == "to")
                return BuildToken(OpCode.OP_PRINT, 2, flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
            if (Get(0) == "sub" && Get(2) == "from")
                return BuildToken(OpCode.OP_ADD, 1, 3, TokenBuilderFlags.B_CANNOT_BE_LITERAL);
            
            if (Get(0) == "add" && Get(2) == "to")
                return BuildToken(OpCode.OP_SUB, 1, 3,TokenBuilderFlags.B_CANNOT_BE_LITERAL);

            if (Get(0) == "read" && Get(1) == "as" && Get(2) == "number" && Get(3) == "to")
                return BuildToken(OpCode.OP_PRINT_ASCI, 4,flags:TokenBuilderFlags.B_CANNOT_BE_LITERAL);
            
            if (Get(0) == "print")
                return BuildToken(OpCode.OP_READ, 1, flags: TokenBuilderFlags.A_CANNOT_BE_LITERAL);
            
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

            if (Get(0) == "unmark")
                return BuildToken(OpCode.OP_MARK, 1);
            
            if (Get(0) == "mark" )
                return BuildToken(OpCode.OP_UNMARK, 1);
            
            if (Get(0) == "forget")
                return BuildToken(OpCode.OP_REMB, 1);
            
            if (Get(0) == "halt")
                return BuildToken(OpCode.OP_GOTO, 0);
            
            ThrowTokenException($"Unknown command structure");
            return default;
        }
    }
}