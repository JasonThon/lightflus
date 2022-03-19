package sheetflow.api;

import com.google.common.collect.BiMap;
import com.google.common.collect.HashBiMap;
import com.google.common.collect.Queues;
import lombok.Getter;
import org.antlr.v4.runtime.tree.ParseTree;
import sheetflow.api.operators.FormulaOp;
import sheetflow.api.operators.FormulaOpBuilder;
import sheetflow.api.utils.NodeIdGenerator;
import sheetflow.common.lang.Lists2;
import sheetflow.common.lang.Objects2;
import sheetflow.syntax.ExprParser;

import java.util.Deque;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Queue;

public class FormulaGraph extends ExprGraph {
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
        List<AdjacentVec<Integer>> meta = Lists2.empty();

        BiMap<FormulaOp, Integer> data = HashBiMap.create();
        NodeIdGenerator generator = new NodeIdGenerator();

        for (AdjacentVec<FormulaOp> adjacentVec : this.meta) {
            AdjacentVec<Integer> adj = new AdjacentVec<>();

            Integer centralId = assignNodeId(data, generator, adjacentVec.getCenter());
            adj.setCenter(centralId);
            data.put(adjacentVec.getCenter(), centralId);

            Lists2.foreach(adjacentVec.getNeighbors(), op -> {
                int nodeId = assignNodeId(data, generator, op);
                adj.addNeighbor(nodeId);
            });

            meta.add(adj);
        }

        return Objects2.toJson(Map.of("meta", meta, "data", data.inverse()));
    }

    private int assignNodeId(final Map<FormulaOp, Integer> data,
                             final NodeIdGenerator generator,
                             final FormulaOp op) {
        int nodeId;

        if (!data.containsKey(op)) {
            nodeId = generator.nextId();
            data.put(op, nodeId);
        } else {
            nodeId = data.get(op);
        }
        return nodeId;
    }
}
