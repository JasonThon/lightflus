package sheetflow.api.operators;

import com.fasterxml.jackson.annotation.JsonIgnore;
import lombok.Getter;
import sheetflow.api.OpType;
import sheetflow.common.lang.Asserts2;
import sheetflow.common.lang.Lists2;

import java.util.List;

@Getter
public abstract class FormulaOp {
    private final OpType type;
    @JsonIgnore
    private List<FormulaOp> upstream;

    protected FormulaOp(final OpType type) {
        Asserts2.notNull(type, "Operator Type is required");
        this.type = type;
    }

    public static FormulaOpBuilder builder() {
        return new FormulaOpBuilder();
    }

    public void addUpStream(final FormulaOp formulaOp) {
        if (this.upstream == null) {
            this.upstream = Lists2.empty();
        }

        this.upstream.add(formulaOp);
    }
}
