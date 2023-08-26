-- CreateTable
CREATE TABLE "Config" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "download_directory" TEXT NOT NULL,
    "download_subdirectory" TEXT NOT NULL,
    "save_file_directory" TEXT NOT NULL,
    "allowed_extensions" TEXT NOT NULL,
    "allowed_origins" TEXT NOT NULL
);
