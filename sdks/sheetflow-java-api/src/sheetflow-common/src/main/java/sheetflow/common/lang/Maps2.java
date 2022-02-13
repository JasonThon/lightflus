package sheetflow.common.lang;

import java.util.Map;

public class Maps2 {
    public static <K, V> boolean isNullOrEmpty(final Map<K, V> map) {
        return map == null || map.isEmpty();
    }
}
