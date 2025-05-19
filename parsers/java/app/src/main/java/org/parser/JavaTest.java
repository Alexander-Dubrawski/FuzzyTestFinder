package org.parser;

public class JavaTest {
    String class_path;
    String method_name;

    public JavaTest() {
    }

    public JavaTest(String classPath, String methodName) {
        this.class_path = classPath;
        this.method_name = methodName;
    }

    public String getClassPath() {
        return class_path;
    }

    public void setClassPath(String classPath) {
        this.class_path = classPath;
    }

    public String getMethodName() {
        return method_name;
    }

    public void setMethodName(String methodName) {
        this.method_name = methodName;
    }
}
