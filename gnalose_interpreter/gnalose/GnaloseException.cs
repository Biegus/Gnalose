using System;

namespace Gnalose
{
    public abstract class GnaloseException : Exception
    {
        public override string Message => $"At line {Line} (line {LineBackw} from the bottom):\n{base.Message}\n(\"{LineString}\")";
        public int Line { get; }
        public string LineString { get; }
        public int LineBackw { get; }
        //line given is NOT reversed
        public GnaloseException(string message,int line,int lineBackw,string lineString)
            :base(message)
        {
            Line = line;
            LineBackw = lineBackw;
            LineString = lineString;
        }
    }
    public class GnaloseInterpreterException: GnaloseException
    {
        public override string Message => $"Failure while running gnalose code:\n{base.Message}";
        
        //line given is NOT reversed
        public GnaloseInterpreterException(string message,int line,int lineBackw,string lineString)
            :base(message,line,lineBackw,lineString)
        {
            
        }
    }
    public class GnaloseTokenizerException: GnaloseException
    {
        public override string Message => $"Failure while tokenizing gnalose code (phase:{Phase}):\n{base.Message}";
        public TokenPhase Phase { get; }
        //line given is NOT reversed
        public GnaloseTokenizerException(string message,int line,int lineBackw, TokenPhase phase, string lineString)
            :base(message,line,lineBackw,lineString)
        {
            Phase = phase;
        }
    }
}