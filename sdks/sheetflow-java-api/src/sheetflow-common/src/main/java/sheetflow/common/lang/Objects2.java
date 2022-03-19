package sheetflow.common.lang;

import com.fasterxml.jackson.core.JsonProcessingException;
import org.springframework.util.Assert;

import java.io.IOException;
import java.io.InputStream;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import static sheetflow.common.lang.Jsons.OBJECT_MAPPER;

public class Objects2 {

    public static <T> String toJson(final T obj) {
        try {
            return OBJECT_MAPPER.writeValueAsString(obj);
        } catch (JsonProcessingException e) {
            throw new RuntimeException(e);
        }
    }

    public static boolean equals(final Object obj1, final Object obj2) {
        return Objects.equals(obj1, obj2);
    }

    public static <T> String asString(final T obj) {
        if (obj == null) {
            return "";
        }

        return Objects.toString(obj);
    }

    public static boolean isNull(Object obj) {
        return obj == null;
    }

    public static <T> Optional<T> fromProperties(final Map<String, Object> properties, final Class<T> tClass) {
        try {
            return Optional.ofNullable(OBJECT_MAPPER.convertValue(properties, tClass));
        } catch (IllegalArgumentException e) {
            return Optional.empty();
        }
    }

    public static <T> Optional<T> fromJsonStream(final InputStream stream, final Class<T> tClass) {
        Assert.notNull(stream, "Stream should not be null");

        try {
            return Optional.ofNullable(OBJECT_MAPPER.readValue(stream, tClass));
        } catch (IOException e) {
            return Optional.empty();
        }
    }


}
