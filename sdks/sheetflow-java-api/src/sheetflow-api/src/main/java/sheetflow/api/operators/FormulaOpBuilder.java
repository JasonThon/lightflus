package sheetflow.api.operators;

import com.google.common.collect.ImmutableMap;
import com.google.common.collect.Maps;
import org.antlr.v4.runtime.Token;
import org.antlr.v4.runtime.tree.ErrorNode;
import org.antlr.v4.runtime.tree.ParseTree;
import org.antlr.v4.runtime.tree.ParseTreeVisitor;
import org.antlr.v4.runtime.tree.RuleNode;
import org.antlr.v4.runtime.tree.TerminalNode;
import org.antlr.v4.runtime.tree.TerminalNodeImpl;
import sheetflow.common.lang.Lists2;
import sheetflow.syntax.ExprParser;

import java.util.AbstractMap;
import java.util.Map;
import java.util.Queue;


public class FormulaOpBuilder implements ParseTreeVisitor<FormulaOp> {

    private static final Map<Class<? extends ParseTree>, ParseTreeProcessor> SUPPORTED_OPERATORS =
            ImmutableMap.copyOf(
                    Lists2.items(
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallSumContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallNowContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallHourContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallMinuteContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallDailyContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(TerminalNodeImpl.class, FormulaOpBuilder::processTerminal),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.AddSubContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.MulDivContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CheckNestedConditionContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CheckConditionContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.IntByAddSubContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.IntByMulDivContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.SimpleFilterExprContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.ComposedSimpleFilterExprContext.class, FormulaOpBuilder::processOperator),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallContainsContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallCountContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallCountIfContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallDistinctCountContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallDistinctCountifContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallDateContext.class, FormulaOpBuilder::processFuncCtx),
                            new AbstractMap.SimpleImmutableEntry<Class<? extends ParseTree>, ParseTreeProcessor>(ExprParser.CallStringConcatContext.class, FormulaOpBuilder::processFuncCtx)
                    )
            );

    private final Map<ParseTree, FormulaOp> treeIndexMap = Maps.newHashMap();
    private Queue<ParseTree> treeQueue;

    private static FormulaOp processOperator(final ParseTree parseTree) {
        if (parseTree instanceof ExprParser.AddSubContext) {
            return OperatorOp.of(((ExprParser.AddSubContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.MulDivContext) {
            return OperatorOp.of(((ExprParser.MulDivContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.CheckNestedConditionContext) {
            return OperatorOp.of(((ExprParser.CheckNestedConditionContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.CheckConditionContext) {
            return OperatorOp.of(((ExprParser.CheckConditionContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.IntByAddSubContext) {
            return OperatorOp.of(((ExprParser.IntByAddSubContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.IntByMulDivContext) {
            return OperatorOp.of(((ExprParser.IntByMulDivContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.SimpleFilterExprContext) {
            return OperatorOp.of(((ExprParser.SimpleFilterExprContext) parseTree).op.getType());
        }

        if (parseTree instanceof ExprParser.ComposedSimpleFilterExprContext) {
            return OperatorOp.of(((ExprParser.ComposedSimpleFilterExprContext) parseTree).op.getType());
        }


        return null;
    }

    private static FormulaOp processFuncCtx(final ParseTree parseTree) {
        return FunctionOp.of(parseTree);
    }

    private static FormulaOp processTerminal(final ParseTree parseTree) {
        TerminalNode terminalNode = (TerminalNode) parseTree;

        Token symbol = terminalNode.getSymbol();
        if (ExprParser.ID == symbol.getType()) {
            return new ReferenceOp(symbol.getText());
        }

        if (ExprParser.STRING == symbol.getType()) {
            return new ValueOp<>(ValueType.String, symbol.getText());
        }

        if (ExprParser.INT == symbol.getType()) {
            return new ValueOp<>(ValueType.Int, Integer.valueOf(symbol.getText()));
        }

        if (ExprParser.DOUBLE == symbol.getType()) {
            return new ValueOp<>(ValueType.Double, Double.valueOf(symbol.getText()));
        }

        return null;
    }

    @Override
    public FormulaOp visit(ParseTree tree) {
        ParseTreeProcessor processor = SUPPORTED_OPERATORS.get(tree.getClass());

        if (processor != null) {
            FormulaOp formulaOp = processor.doProcess(tree, treeIndexMap);
            treeIndexMap.putIfAbsent(tree, formulaOp);

            return formulaOp;
        }

        return null;
    }

    @Override
    public FormulaOp visitChildren(RuleNode node) {
        if (treeQueue != null) {
            for (int i = 0; i < node.getChildCount(); i++) {
                treeQueue.add(node.getChild(i));
            }
        }

        return visit(node);
    }

    @Override
    public FormulaOp visitTerminal(TerminalNode terminalNode) {
        return visit(terminalNode);
    }

    @Override
    public FormulaOp visitErrorNode(ErrorNode node) {
        return null;
    }

    public FormulaOpBuilder treeQueue(Queue<ParseTree> trees) {
        this.treeQueue = trees;
        return this;
    }

    @FunctionalInterface
    private interface ParseTreeProcessor {
        FormulaOp apply(ParseTree parseTree);

        default FormulaOp doProcess(ParseTree parseTree,
                                    Map<ParseTree, FormulaOp> treeIndexMap) {

            FormulaOp formulaOp = apply(parseTree);

            if (treeIndexMap != null) {
                if (formulaOp != null) {
                    ParseTree parent = parseTree.getParent();

                    while (parent != null && !treeIndexMap.containsKey(parent)) {
                        parent = parent.getParent();

                        FormulaOp upstream = treeIndexMap.get(parent);
                        if (parent != null && upstream != null) {
                            formulaOp.addUpStream(upstream);
                            break;
                        }
                    }

                }
            }

            return formulaOp;
        }
    }
}
