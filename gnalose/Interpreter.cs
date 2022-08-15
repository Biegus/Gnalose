using System;
using System.Collections.Generic;
using System.Linq;

namespace Gnalose
{
    public class Interpreter
    {
        public struct OutInfo
        {
          
            public string Out { get; }

            public OutInfo( string @out)
            {
              
                Out = @out;
            }

            public static OutInfo None = default;

            public static OutInfo MakeOut(string val)
            {
                return new OutInfo( val);
            }
        }

        private PreProcessed processed;
        private int line =-1;
        private Dictionary<string, int> variables = new();
        private Dictionary<string, int[]> arrays = new();
        Dictionary<string, int> marks = new();
        private string remembered;
        
        public Interpreter(PreProcessed processed)
        {
            this.processed = processed;
        }



        private int GetLiteralOrRef(UnionRef @ref)
        {
            if (@ref.RefMode)
                return variables[@ref.Reference.Name];
            else
                return @ref.Literal;
        }

        public void RunAll(Action<string> outFunc, Func<int> inFunc)
        {
            while (line+1 < processed.Tokens.Count)
            {
                OutInfo outInfo=  RunNextLine(inFunc);
                if (outInfo.Out!=null)
                    outFunc(outInfo.Out);
            }
        }
       
        public OutInfo RunNextLine(Func<int> inFunc)
        {
            line++;

            void ExecuteMath(Token token,Func<int, int,int> operation)
            {
                int a = GetLiteralOrRef(token.A);
                Reference b = token.B.Reference;
                foreach (var key in variables.Keys.ToArray())
                {
                    if (key == b.Name && b.Index==null) continue;
                    if (token.A.Reference.Name != null && token.A.Reference.Name == key &&
                        token.A.Reference.Index == null) continue;
                    variables[key] = operation(variables[key], a);
                }
                foreach (var key in arrays.Keys)
                {
                    for(int innerKey=0;innerKey<arrays[key].Length;innerKey++)
                    {
                        if (b.Index!=null&&innerKey==GetIndexValue(b.Index.Value)&& key==b.Name) continue;
                        if (token.A.Reference.Name!=null && token.A.Reference.Index!=null&&innerKey==GetIndexValue(token.A.Reference.Index.Value)&& key==token.A.Reference.Name) continue;
                        arrays[key][innerKey] = operation(arrays[key][innerKey], a);
                    }
                }
            }
            int indx = line;
            var token = processed.Tokens[indx];

            int GetValue(UnionRef union)
            {
                if (!union.RefMode)
                    return union.Literal;
                else if (union.Reference.Index == null)
                    return variables[union.Reference.Name];
                else return arrays[union.Reference.Name][GetIndexValue( union.Reference.Index.Value)];
            }

            int GetIndexValue(RefIndex rf)
            {
                if (!rf.RefMode)
                    return rf.Literal;
                else 
                    return variables[rf.Ref];
            }
            void SetValue(UnionRef union, int value)
            {
                if (!union.RefMode)
                    throw new InvalidOperationException("Literal cannot be changed");
                else if (union.Reference.Index == null)
                    variables[union.Reference.Name] = value;
                else  arrays[union.Reference.Name][GetIndexValue( union.Reference.Index.Value)] = value;
            }

            void IfJump(Func<int,int,bool> cond)
            {
                if (!cond(GetLiteralOrRef(token.A),GetLiteralOrRef(token.B)))
                {
                    line = processed.Paths.Dict[line] - 1;
                }
            }
            switch (token.OpCode)
            {
                case OpCode.OP_DEF:
                    variables[token.A.Reference.Name] = 0;
                    return OutInfo.None;
                case OpCode.OP_DEF_A:
                    arrays[token.A.Reference.Name] = new int[token.A.Reference.Index.Value.Literal];
                    break;
                case OpCode.OP_PRINT:
                    return OutInfo.MakeOut(GetValue(token.A).ToString());
                case OpCode.OP_PRINT_ASCI:
                    return OutInfo.MakeOut(((char) GetValue(token.A)).ToString());
                case OpCode.OP_ADD:
                    ExecuteMath(token,(a, b) => a + b);
                    return OutInfo.None;
                case OpCode.OP_SUB:
                    ExecuteMath(token,(a, b) => a - b);
                    return OutInfo.None;
                case OpCode.OP_MULT:
                    ExecuteMath(token,(a, b) => a * b);
                    return OutInfo.None;
                case OpCode.OP_DIV:
                    ExecuteMath(token,(a, b) => a / b);
                    return OutInfo.None;
                case OpCode.OP_READ:
                    int inValue = inFunc();
                    SetValue(token.A,inValue);
                    break;
                case OpCode.OP_IF_E:
                    IfJump((a, b) => a == b);
                    break;
                case OpCode.OP_IF_NE:
                    IfJump((a, b) => a != b);
                    break;
                case OpCode.OP_IF_LE:
                    IfJump((a, b) => a <= b);
                    break;
                case OpCode.OP_IF_GE:
                    IfJump((a, b) => a >= b);
                    break;
                case OpCode.OP_IF_G:
                    IfJump((a, b) => a > b);
                    break;
                case OpCode.OP_IF_L:
                    IfJump((a, b) => a < b);
                    break;
                case OpCode.OP_MARK:
                    marks[token.A.Reference.Name] = line + 1;
                    break;
                case OpCode.OP_REMB:
                    remembered = token.A.Reference.Name;
                    break;
                case OpCode.OP_GOTO:
                    line = marks[remembered] - 1;
                    break;
                case OpCode.OP_UNDEF:
                    variables.Remove(token.A.Reference.Name);
                    break;
                case OpCode.OP_UNDF_AR:
                    arrays.Remove(token.A.Reference.Name);
                    break;
                case OpCode.OP_UNMARK:
                    marks.Remove(token.A.Reference.Name);
                    break;
            }
            if (line == processed.Tokens.Count - 1)
            {
                if (variables.Count > 0 || arrays.Count>0 || marks.Count>0)
                    throw new InvalidOperationException("Some variables or marks are still alive");
            }
            return OutInfo.None;
            
        }
    }
}