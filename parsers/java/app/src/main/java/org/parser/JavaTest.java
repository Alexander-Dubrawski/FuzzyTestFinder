package org.parser;


public class JavaTest {
    String classPath;
    String methodName;

    public JavaTest(String classPath, String methodName) {
        this.classPath = classPath;
        this.methodName = methodName;
    }

    @Override
    public String toString() {
        return "(" + classPath + ", " + methodName + ")";
    }
}