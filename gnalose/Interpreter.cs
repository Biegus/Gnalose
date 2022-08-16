﻿using System;
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

        private TokenCollection tokenCollection;
        private int line =-1;
        private Dictionary<string, int> variables = new();
        private Dictionary<string, int[]> arrays = new();
        Dictionary<string, int> marks = new();
        private string remembered;
        public Interpreter(TokenCollection tokenCollection)
        {
            this.tokenCollection = tokenCollection;
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
            while (line+1 < tokenCollection.Tokens.Count)
            {
                OutInfo outInfo=  RunNextLine(inFunc);
                if (outInfo.Out!=null)
                    outFunc(outInfo.Out);
            }
        }

        public OutInfo RunNextLine(Func<int> inFunc)
        {
            line++;

            void ExecuteMath(Token token, Func<int, int, int> operation)
            {
                int a = GetLiteralOrRef(token.A);
                Reference b = token.B.Reference;
                foreach (var key in variables.Keys.ToArray())
                {
                    if (key == b.Name && b.Index == null) continue;
                    if (token.A.Reference.Name != null && token.A.Reference.Name == key &&
                        token.A.Reference.Index == null) continue;
                    variables[key] = operation(variables[key], a);
                }

                foreach (var key in arrays.Keys)
                {
                    for (int innerKey = 0; innerKey < arrays[key].Length; innerKey++)
                    {
                        if (b.Index != null && innerKey == GetIndexValue(b.Index.Value) && key == b.Name) continue;
                        if (token.A.Reference.Name != null && token.A.Reference.Index != null &&
                            innerKey == GetIndexValue(token.A.Reference.Index.Value) &&
                            key == token.A.Reference.Name) continue;
                        arrays[key][innerKey] = operation(arrays[key][innerKey], a);
                    }
                }
            }

            int indx = line;
            var token = tokenCollection.Tokens[indx];




            void ThrowIfOutOfBounds(string arrayName, int index)
            {
                if (index >= arrays[arrayName].Length || index<0)
                    ThrowInterpreterException($"Array out of bounds. You tried to access index {index}," +
                                              $" while the array length is {arrays[arrayName]}");
            }
            
            void ThrowInterpreterException(string message)
            {
                throw new GnaloseInterpreterException(message, token.OriginalLineNumber,
                    tokenCollection.OriginalLineCount - token.OriginalLineNumber + 1, token.OriginalLine);
            }

            void ThrowVariableNotDefined(string variable)
            {
                ThrowInterpreterException(
                    $"Variable {variable} is not defined, but the code tried to use it");
            }
            void ThrowIfSingleVariableNotDefined(string name)
            {
                if (!variables.ContainsKey(name))
                    ThrowVariableNotDefined(name);
            }

            void ThrowIfArrayNotDefined(string name)
            {
                if (!arrays.ContainsKey(name))
                    ThrowVariableNotDefined(name);
            }
           
            
            int GetValue(UnionRef union)
            {
                if (!union.RefMode)
                    return union.Literal;
                else if (union.Reference.Index == null)
                {
                    ThrowIfSingleVariableNotDefined(union.Reference.Name);
                    return variables[union.Reference.Name];
                }
                else
                {
                    ThrowIfArrayNotDefined(union.Reference.Name);
                    int index = GetIndexValue(union.Reference.Index.Value);
                    ThrowIfOutOfBounds(union.Reference.Name, index);
                    return arrays[union.Reference.Name][index];
                }
            }

            void SetValue(UnionRef union, int value)
            {
                if (!union.RefMode)
                    ThrowInterpreterException("Literal cannot be changed");
                else if (union.Reference.Index == null)
                {
                    ThrowIfSingleVariableNotDefined(union.Reference.Name);
                    variables[union.Reference.Name] = value;
                }
                else
                {
                    ThrowIfArrayNotDefined(union.Reference.Name);
                    int index = GetIndexValue(union.Reference.Index.Value);
                    ThrowIfOutOfBounds(union.Reference.Name, index);
                    arrays[union.Reference.Name][GetIndexValue(union.Reference.Index.Value)] = value;
                }
            }
            int GetIndexValue(RefIndex rf)
            {
                if (!rf.RefMode)
                    return rf.Literal;
                else
                {
                    return GetValue(UnionRef.FromReference(rf.Ref));
                }
            }
            void IfJump(Func<int, int, bool> cond)
            {
                if (!cond(GetLiteralOrRef(token.A), GetLiteralOrRef(token.B)))
                {
                    line = tokenCollection.Paths.Dict[line] - 1;
                }
            }

            OutInfo outInfo = OutInfo.None;
            switch (token.OpCode)
            {
                case OpCode.OP_DEF:
                    variables[token.A.Reference.Name] = 0;
                    break;
                case OpCode.OP_DEF_A:
                    arrays[token.A.Reference.Name] = new int[token.A.Reference.Index.Value.Literal];
                    break;
                case OpCode.OP_PRINT:
                    outInfo = OutInfo.MakeOut(GetValue(token.A).ToString());
                    break;
                case OpCode.OP_PRINT_ASCI:
                    outInfo = OutInfo.MakeOut(((char) GetValue(token.A)).ToString());
                    break;
                case OpCode.OP_ADD:
                    ExecuteMath(token, (a, b) => a + b);
                    return OutInfo.None;
                case OpCode.OP_SUB:
                    ExecuteMath(token, (a, b) => a - b);
                    return OutInfo.None;
                case OpCode.OP_READ:
                    int inValue = inFunc();
                    SetValue(token.A, inValue);
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

            if (line == tokenCollection.Tokens.Count - 1)
            {
                if (variables.Count > 0 || arrays.Count > 0 || marks.Count > 0)
                    ThrowInterpreterException("This is last one line and some variables or marks are still alive.");
            }

            return outInfo;
        }
    }
    
}