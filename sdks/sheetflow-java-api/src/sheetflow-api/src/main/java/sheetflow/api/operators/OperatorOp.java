package sheetflow.api.operators;

import com.google.common.collect.ImmutableMap;
import sheetflow.api.OpType;
import sheetflow.common.lang.Lists2;
import sheetflow.syntax.ExprParser;

import java.util.AbstractMap.SimpleImmutableEntry;
import java.util.Map;

public class OperatorOp extends FormulaOp {

    private static final Map<Integer, OpType> OPERATOR_INDEX_MAP = ImmutableMap.copyOf(
            Lists2.items(
                    new SimpleImmutableEntry<>(ExprParser.ADD, OpType.Add),
                    new SimpleImmutableEntry<>(ExprParser.SUB, OpType.Sub),
                    new SimpleImmutableEntry<>(ExprParser.MUL, OpType.Mul),
                    new SimpleImmutableEntry<>(ExprParser.DIV, OpType.Div),
                    new SimpleImmutableEntry<>(ExprParser.EQ, OpType.Eq),
                    new SimpleImmutableEntry<>(ExprParser.NEQ, OpType.Neq),
                    new SimpleImmutableEntry<>(ExprParser.LT, OpType.Lt),
                    new SimpleImmutableEntry<>(ExprParser.GT, OpType.Gt),
                    new SimpleImmutableEntry<>(ExprParser.LTE, OpType.Lte),
                    new SimpleImmutableEntry<>(ExprParser.GTE, OpType.Gte),
                    new SimpleImmutableEntry<>(ExprParser.AND, OpType.And),
                    new SimpleImmutableEntry<>(ExprParser.OR, OpType.Or)
            )
    );

    protected OperatorOp(final OpType type) {
        super(type);
    }

    public static FormulaOp of(final int type) {
        return new OperatorOp(OPERATOR_INDEX_MAP.get(type));
    }
}
