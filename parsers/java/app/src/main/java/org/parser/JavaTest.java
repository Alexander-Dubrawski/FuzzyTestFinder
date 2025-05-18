package org.parser;


public class JavaTest {
    String classPath;
    String methodName;

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

    public JavaTest(String classPath, String methodName) {
        this.classPath = classPath;
        this.methodName = methodName;
    }
}
