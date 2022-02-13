package sheetflow.syntax.error;

import org.antlr.v4.runtime.ANTLRErrorListener;
import org.antlr.v4.runtime.Parser;
import org.antlr.v4.runtime.RecognitionException;
import org.antlr.v4.runtime.Recognizer;
import org.antlr.v4.runtime.atn.ATNConfigSet;
import org.antlr.v4.runtime.dfa.DFA;

import java.util.BitSet;

public class ExprErrorListener implements ANTLRErrorListener {
    @Override
    public void syntaxError(final Recognizer<?, ?> recognizer,
                            final Object offendingSymbol,
                            final int line,
                            final int charPositionInLine,
                            final String msg,
                            final RecognitionException e) {

    }

    @Override
    public void reportAmbiguity(final Parser recognizer,
                                final DFA dfa,
                                final int startIndex,
                                final int stopIndex,
                                final boolean exact,
                                final BitSet ambigAlts,
                                final ATNConfigSet configs) {

    }

    @Override
    public void reportAttemptingFullContext(final Parser recognizer,
                                            final DFA dfa,
                                            final int startIndex,
                                            final int stopIndex,
                                            final BitSet conflictingAlts,
                                            final ATNConfigSet configs) {

    }

    @Override
    public void reportContextSensitivity(final Parser recognizer,
                                         final DFA dfa,
                                         final int startIndex,
                                         final int stopIndex,
                                         final int prediction,
                                         final ATNConfigSet configs) {

    }
}
