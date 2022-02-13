package sheetflow.api.operators;

import lombok.Getter;
import sheetflow.api.OpType;

@Getter
public class ValueOp<T> extends FormulaOp {
    private ValueType valueType;
    private T value;

    public ValueOp(final ValueType valueType, final T value) {
        this();
        this.value = value;
        this.valueType = valueType;
    }

    ValueOp() {
        super(OpType.Value);
    }
}
