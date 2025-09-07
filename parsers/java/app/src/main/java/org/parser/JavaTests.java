package org.parser;

import spoon.Launcher;
import spoon.reflect.CtModel;
import spoon.reflect.declaration.CtMethod;
import spoon.reflect.declaration.CtType;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.time.Instant;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;

import java.util.logging.Logger;

import com.fasterxml.jackson.annotation.JsonProperty;

public class JavaTests {
    @JsonProperty("root_folder")
    String rootFolder;
    Long timestamp;
    HashMap<String, List<JavaTest>> tests;
    @JsonProperty("failed_tests")
    HashMap<String, List<JavaTest>> failedTests;

    private static final Logger logger = Logger.getLogger(JavaTests.class.getName());

    public JavaTests() {
    }

    public String getRootFolder() {
        return rootFolder;
    }
    public void setRootFolder(String rootFolder) {
        this.rootFolder = rootFolder;
    }

    public Long getTimestamp() {
        return timestamp;
    }
    public void setTimestamp(Long timestamp) {
        this.timestamp = timestamp;
    }

    public HashMap<String, List<JavaTest>> getTests() {
        return tests;
    }
    public void setTests(HashMap<String, List<JavaTest>> tests) {
        this.tests = tests;
    }

    public HashMap<String, List<JavaTest>> getFailedTests() {
        return failedTests;
    }

    public void setFailedTests(HashMap<String, List<JavaTest>> failedTests) {
        this.failedTests = failedTests;
    }

    private Path getRelativePath(Path fullPath) throws Exception {
        try {
            Path relative = Paths.get(rootFolder).relativize(fullPath);

            // Remove leading "/" if present (similar to .strip_prefix("/"))
            String normalized = relative.toString();
            if (normalized.startsWith("/")) {
                normalized = normalized.substring(1);
            }

            return Paths.get(normalized);
        } catch (IllegalArgumentException | NullPointerException e) {
            throw new Exception("File path could not be parsed: " + fullPath);
        }
    }

    public void update() throws IOException {
        filterOutDeletedFiles();
        var rootFolderPath = Paths.get(rootFolder).toAbsolutePath();
        Files.walkFileTree(rootFolderPath, new SimpleFileVisitor<>() {
            @Override
            public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) {
                try {
                    if (!attrs.isRegularFile() || isHidden(file)) {
                        return FileVisitResult.CONTINUE;
                    }

                    String extension = getFileExtension(file);
                    if (!"java".equalsIgnoreCase(extension)) {
                        return FileVisitResult.CONTINUE;
                    }
                    var relativePath = getRelativePath(file);
                    long modifiedTime = attrs.lastModifiedTime().toInstant().toEpochMilli();
                    if (modifiedTime > timestamp) {
                        var newTests = getTestMethodsWithClassPaths(file);
                        if (newTests.isEmpty()) {
                             return FileVisitResult.CONTINUE;
                        }
                        if (!tests.containsKey(relativePath.toString())) {
                            tests.put(relativePath.toString(), newTests);
                        } else {
                            tests.put(relativePath.toString(), newTests);
                        }
                        logger.info("Tests updated: " + relativePath.toString() + " : " + newTests);
                        return FileVisitResult.CONTINUE;
                    }

                    FileTime createdTime = attrs.creationTime();
                    if (createdTime.toInstant().toEpochMilli() > timestamp) {
                        var newTests = getTestMethodsWithClassPaths(relativePath);
                        if (!newTests.isEmpty()) {
                            tests.put(relativePath.toString(), newTests);
                            logger.info("Test created" + relativePath.toString()+ " : " + newTests);
                        }
                        return FileVisitResult.CONTINUE;
                    }
                } catch (Exception e) {
                    e.printStackTrace();
                }
                return FileVisitResult.CONTINUE;
            }
        });
        timestamp = Instant.now().toEpochMilli();
    }    

    private void filterOutDeletedFiles() throws IOException {
        List<String> testToFilterOut = new ArrayList<>();
        for (String strPath : tests.keySet()) {
            var path = Paths.get(rootFolder).resolve(strPath).toAbsolutePath();
            if (!Files.exists(path)) {
                testToFilterOut.add(strPath);
            }
        }
        for (String path : testToFilterOut) {
            logger.info("Remove test: " + path);
            tests.remove(path);
        }
    }

    private static List<JavaTest> getTestMethodsWithClassPaths(Path javaFilePath) throws IOException {
        var absPath = javaFilePath.toAbsolutePath();
        Launcher launcher = new Launcher();
        launcher.addInputResource(absPath.toString());
        launcher.buildModel();

        CtModel model = launcher.getModel();

        List<JavaTest> result = new ArrayList<>();

        for (CtType<?> type : model.getAllTypes()) {
            // Check this type is defined in the target file
            if (type.getPosition().isValidPosition() &&
                type.getPosition().getFile().toPath().equals(absPath)) {

                String classPath = type.getQualifiedName();

                // Get test methods inside this class
                List<CtMethod<?>> testMethods = type.getMethods().stream()
                    .filter(method -> method.getAnnotations().stream()
                        .anyMatch(annotation -> annotation.getAnnotationType().getSimpleName().equals("Test")))
                    .toList();

                for (CtMethod<?> method : testMethods) {
                    result.add(new JavaTest(classPath, method.getSimpleName()));
                }
            }
        }

        return result;
    }
    private boolean isHidden(Path path) throws IOException {
        return Files.isHidden(path);
    }

    private String getFileExtension(Path path) {
        String name = path.getFileName().toString();
        int lastDot = name.lastIndexOf('.');
        return (lastDot == -1) ? "" : name.substring(lastDot + 1);
    }
}
