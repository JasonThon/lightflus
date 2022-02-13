package sheetflow.api;

import com.google.common.collect.Queues;
import lombok.Getter;
import org.antlr.v4.runtime.tree.ParseTree;
import sheetflow.api.operators.FormulaOp;
import sheetflow.api.operators.FormulaOpBuilder;
import sheetflow.common.lang.Lists2;
import sheetflow.common.lang.Objects2;
import sheetflow.syntax.ExprParser;

import java.util.Deque;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Queue;

class FormulaGraph extends ExprGraph {
    @Getter
    private List<AdjacentVec<FormulaOp>> meta;

    public FormulaGraph(final ExprParser parser) {
        super(parser);
        this.updownBuild();
    }

    private void updownBuild() {
        Queue<ParseTree> trees = Queues.newLinkedBlockingQueue();
        Deque<FormulaOp> ops = Queues.newArrayDeque();
        FormulaOpBuilder builder = FormulaOp.builder()
                .treeQueue(trees);

        var expr = parser.expr();

        if (!Lists2.isNullOrEmpty(expr.children)) {
            trees.addAll(expr.children);

            while (!trees.isEmpty()) {
                ParseTree parseTree = trees.poll();

                FormulaOp op = parseTree.accept(builder);

                if (!Objects.isNull(op)) {
                    ops.addFirst(op);
                }
            }
        }

        this.bottomUpBuild(ops);
    }

    private void bottomUpBuild(final Deque<FormulaOp> ops) {
        if (this.meta == null) {
            this.meta = Lists2.empty();
        }

        Lists2.foreach(ops, op -> this.meta.add(new AdjacentVec<>(op, op.getUpstream())));
    }

    @Override
    public String toString() {
        return Objects2.toJson(Map.of("meta", this.meta));
    }
}
