package sheetflow.api;

import org.junit.jupiter.api.Test;
import sheetflow.api.operators.FormulaOp;
import sheetflow.common.lang.Lists2;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

class ExprBuilderTest {
    @Test
    void test_build_single_sum() {
        FormulaGraph testGraph = ExprBuilder.build("sum(AA1)", FormulaGraph.class);
        assertNotNull(testGraph);

        assertFalse(Lists2.isNullOrEmpty(testGraph.getMeta()));
        assertEquals(2, Lists2.size(testGraph.getMeta()));

        List<AdjacentVec<FormulaOp>> filter = Lists2.filter(testGraph.getMeta(), vec -> vec.getCentral().getType().equals(OpType.Reference));
        assertEquals(1, Lists2.size(filter));
        Lists2.foreach(filter, vec -> Lists2.foreach(vec.getNeighbor(), op -> assertEquals(OpType.Sum, op.getType())));
    }

    @Test
    void test_sum_add_num_const() {
        FormulaGraph graph = ExprBuilder.build("sum(AA2) + 1", FormulaGraph.class);

        assertNotNull(graph);
    }
}