package sheetflow.api;

import org.antlr.v4.runtime.ANTLRInputStream;
import org.antlr.v4.runtime.CommonTokenStream;
import sheetflow.syntax.ExprLexer;
import sheetflow.syntax.ExprParser;
import sheetflow.syntax.error.ExprErrorListener;

import java.lang.reflect.InvocationTargetException;

public class ExprBuilder {

    public static <T extends ExprGraph> T build(final String expr, final Class<T> clazz) {
        ExprParser parser = new ExprParser(new CommonTokenStream(new ExprLexer(new ANTLRInputStream(expr))));
        parser.addErrorListener(new ExprErrorListener());

        try {
            return clazz.getDeclaredConstructor(ExprParser.class)
                    .newInstance(parser);
        } catch (InstantiationException | IllegalAccessException | InvocationTargetException | NoSuchMethodException e) {
            throw new RuntimeException(e);
        }
    }
}
