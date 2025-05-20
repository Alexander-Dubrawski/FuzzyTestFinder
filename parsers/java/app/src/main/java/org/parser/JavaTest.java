package org.parser;

import com.fasterxml.jackson.annotation.JsonProperty;

public class JavaTest {
    @JsonProperty("class_path")
    String classPath;
    @JsonProperty("method_name")
    String methodName;

    public JavaTest() {
    }

    public JavaTest(String classPath, String methodName) {
        this.classPath = classPath;
        this.methodName = methodName;
    }

    public String getClassPath() {
        return classPath;
    }

    public void setClassPath(String classPath) {
        this.classPath = classPath;
    }

    public String getMethodName() {
        return methodName;
    }

    public void setMethodName(String methodName) {
        this.methodName = methodName;
    }
}
