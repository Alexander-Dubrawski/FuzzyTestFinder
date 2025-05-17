package org.parser;

import spoon.Launcher;
import spoon.reflect.CtModel;
import spoon.reflect.code.CtBlock;
import spoon.reflect.declaration.CtMethod;
import spoon.reflect.declaration.CtType;
import spoon.reflect.declaration.ModifierKind;
import spoon.reflect.visitor.filter.TypeFilter;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.Stream;

public class Parser {
    private final Path rootFolder;
    private final long timestamp; // Milliseconds since epoch

    public Parser(String rootFolder, long timestamp) {
        this.rootFolder = Paths.get(rootFolder);
        this.timestamp = timestamp;
    }

    public static List<JavaTest> getTestMethodsWithClassPaths(Path javaFilePath) throws IOException {
        Launcher launcher = new Launcher();
        launcher.addInputResource(javaFilePath.toString());
        launcher.buildModel();

        CtModel model = launcher.getModel();

        List<JavaTest> result = new ArrayList<>();

        for (CtType<?> type : model.getAllTypes()) {
            // Check this type is defined in the target file
            if (type.getPosition().isValidPosition() &&
                type.getPosition().getFile().toPath().equals(javaFilePath)) {

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

    public void scan() throws IOException {
        Files.walkFileTree(rootFolder, new SimpleFileVisitor<>() {
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
                    System.out.println("File: " + file.toString());
                    System.out.println(getTestMethodsWithClassPaths(file));
                    long modifiedTime = attrs.lastModifiedTime().toInstant().toEpochMilli();
                    if (modifiedTime > timestamp) {
                        System.out.println("Tests updated: " + file.toString());
                    }

                    FileTime createdTime = attrs.creationTime();
                    if (createdTime.toInstant().toEpochMilli() > timestamp) {
                        System.out.println("Created");
                    }

                } catch (Exception e) {
                    e.printStackTrace();
                }
                return FileVisitResult.CONTINUE;
            }
        });
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