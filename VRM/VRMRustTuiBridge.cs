#if UNITY_EDITOR
using System;
using System.Collections.Generic;
using System.IO;
using UnityEditor;
using UnityEngine;

public class VRMRustTuiBridgeWindow : EditorWindow
{
    [Serializable]
    private class BoneInfo
    {
        public string name;
        public string parent_name;
    }

    [Serializable]
    private class SmrInfo
    {
        public string name;
        public string root_bone;
        public List<string> bones = new List<string>();
    }

    [Serializable]
    private class VrmInputPayload
    {
        public List<BoneInfo> avatar_bones = new List<BoneInfo>();
        public List<BoneInfo> clothing_bones = new List<BoneInfo>();
        public List<SmrInfo> clothing_smrs = new List<SmrInfo>();
        public string suffix = "";
    }

    [Serializable]
    private class VrmReportPayload
    {
        public List<string> duplicate_avatar_bone_names = new List<string>();
        public List<string> duplicate_clothing_bone_names = new List<string>();
        public int referenced_clothing_bones;
        public int estimated_moved_bones;
        public int estimated_moved_smrs;
        public int estimated_renamed_bones;
        public int estimated_renamed_smrs;
        public List<string> warnings = new List<string>();
    }

    [SerializeField] private GameObject m_avatarRoot;
    [SerializeField] private GameObject m_clothingRoot;
    [SerializeField] private string m_suffix = "";
    [SerializeField] private string m_rustProjectPath = "tools/vrm-tui";
    [SerializeField] private string m_inputPath = "Temp/vrm_tui_input.json";
    [SerializeField] private string m_outputPath = "Temp/vrm_tui_report.json";

    [MenuItem("Tools/VRM/Rust TUI 브리지")]
    private static void Open()
    {
        GetWindow<VRMRustTuiBridgeWindow>("Rust TUI 브리지");
    }

    private void OnGUI()
    {
        EditorGUILayout.LabelField("Rust TUI 브리지", EditorStyles.boldLabel);
        EditorGUILayout.Space(4);

        m_avatarRoot = (GameObject)EditorGUILayout.ObjectField("아바타 루트", m_avatarRoot, typeof(GameObject), true);
        m_clothingRoot = (GameObject)EditorGUILayout.ObjectField("의상 루트", m_clothingRoot, typeof(GameObject), true);
        m_suffix = EditorGUILayout.TextField("접미사", m_suffix);
        EditorGUILayout.Space(4);

        m_rustProjectPath = EditorGUILayout.TextField("Rust 프로젝트 경로", m_rustProjectPath);
        m_inputPath = EditorGUILayout.TextField("입력 JSON 경로", m_inputPath);
        m_outputPath = EditorGUILayout.TextField("출력 JSON 경로", m_outputPath);

        EditorGUILayout.Space(10);
        if (GUILayout.Button("1) 입력 JSON 내보내기", GUILayout.Height(24)))
        {
            ExportInputJson();
        }

        if (GUILayout.Button("2) TUI 실행 명령 복사", GUILayout.Height(24)))
        {
            CopyRunCommand();
        }

        if (GUILayout.Button("3) 결과 JSON 불러오기", GUILayout.Height(24)))
        {
            LoadReportJson();
        }

        EditorGUILayout.Space(8);
        EditorGUILayout.HelpBox(
            "권장 흐름: 1) 입력 JSON 내보내기 -> 2) 실행 명령 복사 후 터미널 실행 -> 3) TUI에서 Enter로 분석 후 s로 저장 -> 4) 결과 JSON 불러오기",
            MessageType.Info);
    }

    private void ExportInputJson()
    {
        if (m_avatarRoot == null || m_clothingRoot == null)
        {
            EditorUtility.DisplayDialog("Rust TUI 브리지", "아바타 루트와 의상 루트를 모두 지정하세요.", "OK");
            return;
        }

        var payload = BuildPayload();
        var json = JsonUtility.ToJson(payload, true);
        var absPath = ToAbsolutePath(m_inputPath);
        Directory.CreateDirectory(Path.GetDirectoryName(absPath) ?? ".");
        File.WriteAllText(absPath, json);

        AssetDatabase.Refresh();
        EditorUtility.DisplayDialog("Rust TUI 브리지", $"입력 파일 저장 완료\n{absPath}", "OK");
    }

    private void CopyRunCommand()
    {
        var inputAbs = ToAbsolutePath(m_inputPath);
        var outputAbs = ToAbsolutePath(m_outputPath);
        var projectAbs = ToAbsolutePath(m_rustProjectPath);
        var cargoToml = Path.Combine(projectAbs, "Cargo.toml").Replace("\\", "/");

        var cmd = "NIX_CONFIG='experimental-features = nix-command flakes' " +
                  $"nix develop -c cargo run --manifest-path '{cargoToml}' -- " +
                  $"--input '{inputAbs.Replace("\\", "/")}' --output '{outputAbs.Replace("\\", "/")}'";

        EditorGUIUtility.systemCopyBuffer = cmd;
        EditorUtility.DisplayDialog("Rust TUI 브리지", "실행 명령을 클립보드에 복사했습니다.", "OK");
    }

    private void LoadReportJson()
    {
        var absPath = ToAbsolutePath(m_outputPath);
        if (!File.Exists(absPath))
        {
            EditorUtility.DisplayDialog("Rust TUI 브리지", $"결과 파일이 없습니다.\n{absPath}", "OK");
            return;
        }

        var json = File.ReadAllText(absPath);
        var report = JsonUtility.FromJson<VrmReportPayload>(json);
        if (report == null)
        {
            EditorUtility.DisplayDialog("Rust TUI 브리지", "결과 JSON 파싱에 실패했습니다.", "OK");
            return;
        }

        var warningText = "\n- 없음";
        if (report.warnings != null && report.warnings.Count > 0)
        {
            warningText = "\n- " + string.Join("\n- ", report.warnings);
        }

        var summary =
            $"중복(아바타): {report.duplicate_avatar_bone_names.Count}\n" +
            $"중복(의상): {report.duplicate_clothing_bone_names.Count}\n" +
            $"참조 의상 본: {report.referenced_clothing_bones}\n" +
            $"예상 이동 본: {report.estimated_moved_bones}\n" +
            $"예상 이동 SMR: {report.estimated_moved_smrs}\n" +
            $"예상 이름변경 본: {report.estimated_renamed_bones}\n" +
            $"예상 이름변경 SMR: {report.estimated_renamed_smrs}\n" +
            $"경고:{warningText}";

        Debug.Log("[Rust TUI 브리지] 결과 요약\n" + summary);
        EditorUtility.DisplayDialog("Rust TUI 결과", summary, "OK");
    }

    private VrmInputPayload BuildPayload()
    {
        var payload = new VrmInputPayload
        {
            suffix = m_suffix ?? ""
        };

        payload.avatar_bones = CollectBones(m_avatarRoot.transform);
        payload.clothing_bones = CollectBones(m_clothingRoot.transform);
        payload.clothing_smrs = CollectSmrs(m_clothingRoot.transform);
        return payload;
    }

    private static List<BoneInfo> CollectBones(Transform root)
    {
        var list = new List<BoneInfo>();
        foreach (var t in root.GetComponentsInChildren<Transform>(true))
        {
            if (t == null) continue;
            list.Add(new BoneInfo
            {
                name = t.name,
                parent_name = t.parent != null ? t.parent.name : null
            });
        }
        return list;
    }

    private static List<SmrInfo> CollectSmrs(Transform root)
    {
        var list = new List<SmrInfo>();
        var smrs = root.GetComponentsInChildren<SkinnedMeshRenderer>(true);
        foreach (var smr in smrs)
        {
            if (smr == null) continue;

            var info = new SmrInfo
            {
                name = smr.name,
                root_bone = smr.rootBone != null ? smr.rootBone.name : null
            };

            if (smr.bones != null)
            {
                for (var i = 0; i < smr.bones.Length; i++)
                {
                    var bone = smr.bones[i];
                    if (bone != null)
                    {
                        info.bones.Add(bone.name);
                    }
                }
            }

            list.Add(info);
        }
        return list;
    }

    private static string ToAbsolutePath(string path)
    {
        if (string.IsNullOrWhiteSpace(path))
        {
            return Path.GetFullPath(".");
        }

        if (Path.IsPathRooted(path))
        {
            return path;
        }

        var cwd = Directory.GetCurrentDirectory();
        return Path.GetFullPath(Path.Combine(cwd, path));
    }
}
#endif
