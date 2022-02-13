package sheetflow.common.lang;

public class Asserts2 {

    public static void notNull(final Object obj, final String message) {
        if (obj == null) {
            throw new NullPointerException(message);
        }
    }
}
