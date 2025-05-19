package org.parser;

import java.io.IOException;

import org.apache.commons.cli.*;


public class App {


    public static void main(String[] args) {
        Options options = new Options();
        options.addOption("p", "path", true, "Path to JAVA project");
        options.addOption("c", "cache", true, "Path to FzT JAVA project cache file");

        CommandLineParser parser = new DefaultParser();
        CommandLine cmd;
        String path = null;
        String cache = null;

        try {
             cmd = parser.parse(options, args);
             if (cmd.hasOption("path")) {
                path = cmd.getOptionValue("path");
             } else {
                path = System.getenv("JAVA_PROJECT_PATH");
             }
             if (cmd.hasOption("cache")) {
                cache = cmd.getOptionValue("cache");
             } else {
                cache = System.getenv("JAVA_PROJECT_FZT_CACHE_PATH");
             }
            if (path == null) {
                System.err.println("Missing --path option or JAVA_PROJECT_PATH env variable.");
                System.exit(1);
            }
        } catch (ParseException e) {
            System.err.println("Error parsing command line: " + e.getMessage());
            System.exit(1);
        }
        var testParser = new Parser();
        try {
            var result = testParser.parse(path, cache);
            System.out.println(result);
        } catch (IOException e) {
            System.err.println("Error parsing java tests: " + e.getMessage());
            System.exit(1);
        }
    }
}
