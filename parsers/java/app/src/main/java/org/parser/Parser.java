package org.parser;

import java.util.HashMap;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

public class Parser {

    public void parse(String projectPath, String cachePath) throws IOException {
        JavaTests javaTests;
        if (cachePath == null) {
            javaTests = new JavaTests();
            javaTests.setRootFolder(projectPath);
            javaTests.setTimestamp(0L);
            javaTests.setTests(new HashMap<>());
        }  else {
            final ObjectMapper objectMapper = new ObjectMapper();
            var jsonContent = Files.readString(Paths.get(cachePath));
            javaTests = objectMapper.readValue(jsonContent, new TypeReference<>() {
            });
        }
        javaTests.update();
    }
}