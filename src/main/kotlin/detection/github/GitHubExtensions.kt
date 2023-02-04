package detection.github

import org.kohsuke.github.GHRelease

object GitHubExtensions {
    /**
     * Extracts formatted release notes from a given release.
     *
     * 1. The function first splits the release notes body into lines and cleans each line by removing dropdowns,
     * changing all bullet points to be dashes, removing code formatted with backticks, and converting Markdown links
     * to plaintext.
     * 2. It then uses a buildString block to loop through each line of the release notes.
     * 3. If the line starts with "#" and there is another bullet point within two lines of it, it is added.
     * 4. If the line starts with "- " it is added, with each sentence being on a new line and indented.
     * 5. Finally, either the string is returned, or null if it is blank.
     *
     * @param release the [GHRelease] object containing the release notes to be formatted
     * @return A formatted string of the release notes or null if the release notes are blank
     */
    fun getFormattedReleaseNotes(release: GHRelease): String? {
        val lines = release.body
            ?.replace(Regex("<details>.*?</details>", setOf(RegexOption.DOT_MATCHES_ALL, RegexOption.IGNORE_CASE)), "")
            ?.lines()
            ?.map { line ->
                line.trim()
                    .let { if (it.startsWith("* ")) it.replaceFirst("* ", "- ") else it }
                    .replace(Regex("""~+([^~]+)~+"""), "$1")
                    .replace(Regex("""\*+([^*]+)\*+"""), "$1")
                    .replace("`", "")
                    .replace(Regex("\\[([^]]+)]\\([^)]+\\)"), "$1")
            }
        return buildString {
            lines?.forEachIndexed { index, line ->
                when {
                    line.startsWith("#") -> {
                        if (
                            lines.getOrNull(index + 1)?.startsWith("- ") == true ||
                            lines.getOrNull(index + 2)?.startsWith("- ") == true
                        ) {
                            line.dropWhile { it == '#' }.trim().takeUnless { it.isBlank() }?.let { appendLine(it) }
                        }
                    }
                    line.startsWith("- ") -> {
                        appendLine(
                            "- ${line.replace(Regex("([A-Z][a-z].*?[.:!?](?=\$| [A-Z]))"), "$1\n ").drop(2).trim()}"
                        )
                    }
                }
            }
        }.trim().ifBlank { null }
    }
}