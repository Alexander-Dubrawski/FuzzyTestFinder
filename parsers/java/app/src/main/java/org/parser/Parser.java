package org.parser;

import java.util.HashMap;
import java.io.IOException;

public class Parser {

    // Move to Java Tests
    public void parse(String projectPath, String cachePath) throws IOException {
        var javaTests = new JavaTests();
        javaTests.setRootFolder(projectPath);
        javaTests.setTimestamp(0L);
        javaTests.setTests(new HashMap<>());
        javaTests.update();
    }
}