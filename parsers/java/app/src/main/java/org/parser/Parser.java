package org.parser;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.nio.file.attribute.FileTime;
import java.util.stream.Stream;

public class Parser {
    private final Path rootFolder;
    private final long timestamp; // Milliseconds since epoch

    public Parser(String rootFolder, long timestamp) {
        this.rootFolder = Paths.get(rootFolder);
        this.timestamp = timestamp;
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