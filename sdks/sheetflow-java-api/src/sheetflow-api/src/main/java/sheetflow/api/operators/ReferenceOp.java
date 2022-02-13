package sheetflow.api.operators;

import lombok.Getter;
import sheetflow.api.OpType;

public class ReferenceOp extends FormulaOp {
    @Getter
    private String ref;

    public ReferenceOp(final String ref) {
        this();
        this.ref = ref;
    }

    ReferenceOp() {
        super(OpType.Reference);
    }
}
