package data

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch
import ktor.Ktor
import org.kohsuke.github.GHContent
import org.koin.core.annotation.Single
import org.koin.core.component.KoinComponent
import org.koin.core.component.get
import schemas.manifest.DefaultLocaleManifest
import schemas.manifest.InstallerManifest
import schemas.manifest.LocaleManifest
import schemas.manifest.VersionManifest

@Single
class PreviousManifestData : KoinComponent {
    var sharedManifestData: SharedManifestData = get()
    var remoteInstallerData: InstallerManifest? = null
    private val githubImpl = get<GitHubImpl>()
    private val repository = githubImpl.getMicrosoftWingetPkgs()
    private val directoryPath: MutableList<GHContent>? = sharedManifestData.latestVersion?.let {
        repository?.getDirectoryContent("${Ktor.getDirectoryPath(sharedManifestData.packageIdentifier)}/$it")
    }
    var remoteInstallerDataJob: Job = CoroutineScope(Dispatchers.IO).launch {
        directoryPath?.let { nonNullDirectoryPath ->
            repository?.getFileContent(
                nonNullDirectoryPath.first { it.name == "${sharedManifestData.packageIdentifier}.installer.yaml" }.path
            )?.read()?.use {
                remoteInstallerData = YamlConfig.default.decodeFromStream(InstallerManifest.serializer(), it)
            }
        }
    }
    var remoteVersionDataJob: Job = CoroutineScope(Dispatchers.IO).launch(Dispatchers.IO) {
        directoryPath?.let { nonNullDirectoryPath ->
            repository?.getFileContent(
                nonNullDirectoryPath.first { it.name == "${sharedManifestData.packageIdentifier}.yaml" }.path
            )?.read()?.use { remoteVersionData = YamlConfig.default.decodeFromStream(VersionManifest.serializer(), it) }
        }
    }.also { job ->
        job.invokeOnCompletion {
            remoteVersionData?.defaultLocale?.let { sharedManifestData.defaultLocale = it }
        }
    }
    var remoteDefaultLocaleData: DefaultLocaleManifest? = null
    var remoteDefaultLocaleDataJob: Job = CoroutineScope(Dispatchers.IO).launch(Dispatchers.IO) {
        remoteVersionDataJob.join()
        directoryPath?.let { nonNullDirectoryPath ->
            repository?.getFileContent(
                nonNullDirectoryPath.first {
                    it.name == "${sharedManifestData.packageIdentifier}.locale.${sharedManifestData.defaultLocale}.yaml"
                }.path
            )?.read()?.use {
                remoteDefaultLocaleData = YamlConfig.default.decodeFromStream(DefaultLocaleManifest.serializer(), it)
            }
        }
    }
    var remoteLocaleData: List<LocaleManifest>? = null
    var remoteLocaleDataJob: Job = CoroutineScope(Dispatchers.IO).launch {
        remoteVersionDataJob.join()
        directoryPath
            ?.filter {
                it.name.matches(Regex("${Regex.escape(sharedManifestData.packageIdentifier)}.locale\\..*\\.yaml"))
            }
            ?.filterNot { it.name.contains(sharedManifestData.defaultLocale) }
            ?.forEach { ghContent ->
                repository?.getFileContent(ghContent.path)
                    ?.read()
                    ?.use {
                        remoteLocaleData = if (remoteLocaleData == null) {
                            listOf(YamlConfig.default.decodeFromStream(LocaleManifest.serializer(), it))
                        } else {
                            remoteLocaleData!! + YamlConfig.default.decodeFromStream(LocaleManifest.serializer(), it)
                        }
                    }
            }
    }
    var remoteVersionData: VersionManifest? = null
}
