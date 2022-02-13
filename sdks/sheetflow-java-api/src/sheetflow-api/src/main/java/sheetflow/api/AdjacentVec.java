package sheetflow.api;

import lombok.Getter;
import lombok.NoArgsConstructor;

import java.util.List;

@Getter
@NoArgsConstructor
public class AdjacentVec<T> {
    private T central;
    private List<T> neighbor;

    public AdjacentVec(final T central,
                       final List<T> neighbor) {
        this.central = central;
        this.neighbor = neighbor;
    }
}
