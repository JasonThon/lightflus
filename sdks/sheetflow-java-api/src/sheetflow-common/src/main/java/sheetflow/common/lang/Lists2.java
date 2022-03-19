package sheetflow.common.lang;

import com.google.common.collect.Lists;

import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Stream;

public class Lists2 {
    public static <T> boolean isNullOrEmpty(Collection<T> children) {
        return children == null || children.size() == 0;
    }

    public static <T> List<T> empty() {
        return Lists.newLinkedList();
    }

    public static <T> List<T> fixed(final int expectedSize) {
        return Lists.newArrayListWithExpectedSize(expectedSize);
    }

    public static <T> Optional<T> last(final List<T> list) {
        if (list == null || list.isEmpty()) {
            return Optional.empty();
        }

        return Optional.of(list.get(list.size() - 1));
    }

    public static <T> List<T> items(final T... items) {
        if (items == null) {
            return empty();
        }

        int length = items.length;
        List<T> list = fixed(length);

        list.addAll(Arrays.asList(items));

        return list;
    }

    public static <T> boolean anyMatch(final List<T> list, final Predicate<T> predicate) {
        for (T t : list) {
            if (predicate.test(t)) {
                return true;
            }
        }

        return false;
    }

    public static <T> void foreach(final Collection<T> items, final Consumer<T> consumer) {
        if (!Lists2.isNullOrEmpty(items)) {
            for (T item : items) {
                consumer.accept(item);
            }
        }
    }

    public static <T> int size(final List<T> list) {
        return isNullOrEmpty(list) ? 0 : list.size();
    }

    public static <T> List<T> filter(final List<T> list, Predicate<T> predicate) {
        List<T> results = empty();

        if (isNullOrEmpty(list)) {
            return results;
        }

        for (T t : list) {
            if (predicate.test(t)) {
                results.add(t);
            }
        }

        return results;
    }

    public static <T> List<T> appendTail(final T item, final List<T> list) {
        if (list == null) {
            return items(item);
        }

        List<T> result = fixed(list.size() + 1);
        result.addAll(list);
        result.add(item);

        return result;
    }

    public static <T> Stream<T> stream(final List<T> args) {
        if (args == null) {
            return Stream.empty();
        }

        return args.stream();
    }

    public static <T, S> List<S> map(final List<T> list, final Function<T, S> mapper) {
        if (list == null || mapper == null) {
            return Lists2.empty();
        }

        List<S> result = Lists2.empty();
        for (T item : list) {
            result.add(mapper.apply(item));
        }

        return result;
    }

    public static <K, V> Map<K, List<V>> group(final List<V> list, final Function<V, K> keyMapper) {
        if (list == null || keyMapper == null) {
            return Maps2.empty();
        }

        Map<K, List<V>> map = Maps2.empty();

        for (V value : list) {
            K key = keyMapper.apply(value);
            if (map.containsKey(key)) {
                map.get(key).add(value);
            } else {
                map.put(key, Lists2.items(value));
            }
        }

        return map;
    }
}
