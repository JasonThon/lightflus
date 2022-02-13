package sheetflow.api.operators;

import org.junit.jupiter.api.Test;
import sheetflow.api.OpType;

import static org.junit.jupiter.api.Assertions.assertNotEquals;

class FormulaOpTest {
    @Test
    void test_hashcode() {
        FormulaOp sumOp1 = new FunctionOp(OpType.Sum);
        FormulaOp sumOp2 = new FunctionOp(OpType.Sum);

        assertNotEquals(sumOp1.hashCode(), sumOp2.hashCode());
    }
}