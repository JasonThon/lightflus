package sheetflow.api;

import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.Setter;
import sheetflow.common.lang.Lists2;

import java.util.List;

@Getter
@NoArgsConstructor
public class AdjacentVec<T> {
    @Setter
    private T center;
    private List<T> neighbors;

    public AdjacentVec(final T center,
                       final List<T> neighbors) {
        this.center = center;
        this.neighbors = neighbors;
    }

    public void addNeighbor(final T neighbor) {
        if (this.neighbors == null) {
            this.neighbors = Lists2.empty();
        }

        this.neighbors.add(neighbor);
    }
}