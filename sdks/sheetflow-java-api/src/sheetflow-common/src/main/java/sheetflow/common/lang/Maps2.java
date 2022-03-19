package sheetflow.common.lang;

import com.google.common.collect.Maps;

import java.util.Map;

public class Maps2 {
    public static <K, V> boolean isNullOrEmpty(final Map<K, V> map) {
        return map == null || map.isEmpty();
    }

    public static <K, V> Map<K, V> empty() {
        return Maps.newHashMap();
    }
}
