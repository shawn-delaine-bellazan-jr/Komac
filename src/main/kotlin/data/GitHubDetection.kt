package data

import io.ktor.http.Url
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.launch
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import org.koin.core.component.inject
import schemas.SchemasImpl
import java.time.LocalDate
import java.time.ZoneId

class GitHubDetection(url: Url) : KoinComponent {
    var publisherUrl: Deferred<Url?>? = null
    var shortDescription: Deferred<String?>? = null
    var publisherSupportUrl: Deferred<Url?>? = null
    var license: Deferred<String?>? = null
    var licenseUrl: Deferred<Url?>? = null
    var packageUrl: Deferred<Url?>? = null
    var releaseDate: Deferred<LocalDate?>? = null
    var releaseNotesUrl: Deferred<Url?>? = null
    var releaseNotes: Deferred<String?>? = null
    var privacyUrl: Deferred<Url?>? = null
    var topics: Deferred<List<String>?>? = null

    private val githubImpl: GitHubImpl by inject()

    init {
        require(url.host.equals(other = gitHubWebsite, ignoreCase = true)) { "Url must be a GitHub Url" }
        CoroutineScope(Dispatchers.IO).launch {
            val tag = url.pathSegments.dropLast(1).last()
            val repository = githubImpl.github.getRepository("${url.pathSegments[1]}/${url.pathSegments[2]}")
            val release = repository.getReleaseByTagName(tag)
            val asset = release.listAssets().first { it.browserDownloadUrl == url.toString() }
            releaseDate = async { LocalDate.ofInstant(asset.createdAt.toInstant(), ZoneId.systemDefault()) }
            license = async { repository.license.key.uppercase() }
            packageUrl = async { Url(repository.htmlUrl.toURI()) }
            licenseUrl = async { Url(repository.licenseContent.htmlUrl) }
            privacyUrl = async {
                repository
                    .getDirectoryContent("")
                    .find { it.name.lowercase().contains("privacy") }
                    ?.htmlUrl
                    ?.let { Url(it) }
            }
            releaseNotesUrl = async { Url(release.htmlUrl.toURI()) }
            releaseNotes = async {
                buildString {
                    release.body.lineSequence().forEach {
                        if (it.startsWith("* ") || it.startsWith("- ") || it.startsWith("#")) {
                            appendLine(it.replace("#", "").trim())
                        }
                    }
                }.replace("* ", "- ").replace(Regex("\\[([^\\]]+)\\]\\([^\\)]+\\)"), "$1").trim().ifBlank { null }
            }
            topics = async { repository.listTopics() }
            publisherUrl = async { runCatching { repository.owner.blog }.getOrNull()?.let { Url(it) } }
            shortDescription = async { repository.description }
            publisherSupportUrl = async {
                data.shared.Url.isUrlValid(
                    url = "$url/support",
                    schema = get<SchemasImpl>().defaultLocaleSchema,
                    canBeBlank = false
                ).let { error ->
                    if (error == null) {
                        "$url/support"
                    } else {
                        if (repository.hasIssues()) "https://github.com/${repository.fullName}/issues" else null
                    }?.let { Url(it) }
                }
            }
        }
    }

    companion object {
        const val gitHubWebsite = "github.com"
    }
}