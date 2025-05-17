package org.parser;

import java.util.HashMap;
import java.util.List;

public class JavaTests {
    String rootFolder;
    Integer timestamp;
    HashMap<String, List<JavaTest>> tests;

    public String getRootFolder() {
        return rootFolder;
    }
    public void setRootFolder(String rootFolder) {
        this.rootFolder = rootFolder;
    }

    public Integer getTimestamp() {
        return timestamp;
    }
    public void setTimestamp(Integer timestamp) {
        this.timestamp = timestamp;
    }

    public HashMap<String, List<JavaTest>> getTests() {
        return tests;
    }
    public void setTests(HashMap<String, List<JavaTest>> tests) {
        this.tests = tests;
    }
}