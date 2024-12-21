# Program for Smart Generation and Verification of File Checksums

Main Functionalities:

+ 'Generate': Generates DD files (Directory Digest - a checksum file for directories) for a specified directory
+ 'Check': Verifies checksums for a directory using DD (with an option to save a new DD file if errors are found)
+ 'Fast refresh': Updates the DD file by generating checksums only for files that:
    - Are not in DD because they:
        - Are new
        - Were moved (found using checksum - their path in DD will be corrected)
    - Were modified (detected using modification date and file size)
+ 'Full refresh': Functions like fast refresh but calculates checksums for all files

File Information Stored:

- Path
- Checksum
- Modification date
- File size

Usage: This program uses a console-based user interface.

## TODO:

- Implement calculation of checksums for files older than a specified time/date in fast refresh
- Add option to save checksums after completing the 'Check' process
- Add support for creation date tracking
- In fast and full refresh modes, list only files with creation or modification dates older/newer than the digest file's
  creation date