package sheetflow.api.operators;

import com.google.common.collect.ImmutableMap;
import org.antlr.v4.runtime.tree.ParseTree;
import sheetflow.api.OpType;
import sheetflow.common.lang.Lists2;
import sheetflow.syntax.ExprParser;

import java.util.AbstractMap;
import java.util.Map;
import java.util.Optional;
import java.util.function.Supplier;

public class FunctionOp extends FormulaOp {
    private static final Map<Class<? extends ParseTree>, Supplier<FunctionOp>> FUNCTION_OP_BUILDER = ImmutableMap.copyOf(
            Lists2.items(
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallSumContext.class, () -> new FunctionOp(OpType.Sum)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallNowContext.class, () -> new FunctionOp(OpType.Now)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallHourContext.class, () -> new FunctionOp(OpType.Hour)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallMinuteContext.class, () -> new FunctionOp(OpType.Minute)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallDailyContext.class, () -> new FunctionOp(OpType.Daily)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallContainsContext.class, () -> new FunctionOp(OpType.Contains)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallCountContext.class, () -> new FunctionOp(OpType.Count)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallCountIfContext.class, () -> new FunctionOp(OpType.CountIf)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallDistinctCountContext.class, () -> new FunctionOp(OpType.DistinctCount)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallDistinctCountifContext.class, () -> new FunctionOp(OpType.DistinctCountif)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallDateContext.class, () -> new FunctionOp(OpType.Date)),
                    new AbstractMap.SimpleImmutableEntry<>(ExprParser.CallStringConcatContext.class, () -> new FunctionOp(OpType.StringConcat))
            )
    );

    public FunctionOp(final OpType opType) {
        super(opType);
    }

    public static FunctionOp of(final ParseTree parseTree) {
        return Optional.ofNullable(FUNCTION_OP_BUILDER.get(parseTree.getClass()))
                .map(Supplier::get)
                .orElse(null);
    }
}
