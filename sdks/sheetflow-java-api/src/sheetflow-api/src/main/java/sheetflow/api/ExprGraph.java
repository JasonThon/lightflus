package sheetflow.api;

import sheetflow.common.lang.Asserts2;
import sheetflow.syntax.ExprParser;

import java.io.Serializable;

public abstract class ExprGraph implements Serializable {
    protected transient final ExprParser parser;

    protected ExprGraph(ExprParser parser) {
        Asserts2.notNull(parser, "ExprParser should not be null");
        this.parser = parser;
    }
}
