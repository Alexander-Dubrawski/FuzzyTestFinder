package org.parser;

import java.util.HashMap;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.ObjectWriter;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

import java.util.logging.Logger;

public class Parser {
    private static final Logger logger = Logger.getLogger(Parser.class.getName());

    public String parse(String projectPath, String cacheJson) throws IOException {
        JavaTests javaTests;
        if (cacheJson == null) {
            javaTests = new JavaTests();
            javaTests.setRootFolder(projectPath);
            javaTests.setTimestamp(0L);
            javaTests.setTests(new HashMap<>());
        }  else {
            final ObjectMapper objectMapper = new ObjectMapper();
            javaTests = objectMapper.readValue(cacheJson, new TypeReference<>() {
            });
            if (!javaTests.rootFolder.equals(projectPath)) {
                logger.warning("Root Folders are different. Cache: " + javaTests.rootFolder + " != Project path : " + projectPath + " Root folder set to project folder");
                javaTests.rootFolder = projectPath;
            }
        }
        javaTests.update();
        final ObjectWriter ow = new ObjectMapper().writer().withDefaultPrettyPrinter();
        return ow.writeValueAsString(javaTests);
    }
}
