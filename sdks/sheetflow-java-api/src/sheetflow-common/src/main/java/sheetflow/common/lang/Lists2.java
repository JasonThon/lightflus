package sheetflow.common.lang;

import com.google.common.collect.Lists;

import java.util.Arrays;
import java.util.Collection;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Predicate;

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

    public static <T> List<T> items(final T... items) {
        if (items == null) {
            return empty();
        }

        int length = items.length;
        List<T> list = Lists.newArrayListWithExpectedSize(length);

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
}
