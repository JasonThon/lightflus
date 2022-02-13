package sheetflow.common.lang;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.util.Objects;

public class Objects2 {
    private static final ObjectMapper objectMapper = new ObjectMapper();

    public static <T> String toJson(final T obj) {
        try {
            return objectMapper.writeValueAsString(obj);
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }

    public static boolean equals(final Object obj1, final Object obj2) {
        return Objects.equals(obj1, obj2);
    }
}
