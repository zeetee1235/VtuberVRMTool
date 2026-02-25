#if UNITY_EDITOR
using System;
using System.Collections.Generic;
using System.Linq;
using UnityEditor;
using UnityEngine;

public class VRMClothingMergerWindow : EditorWindow
{
    [SerializeField] private GameObject m_avatarRoot;
    [SerializeField] private GameObject m_clothingRoot;
    [SerializeField] private string m_name = "";

    [MenuItem("Tools/VRM/옷 입히는 툴")]
    private static void Open()
    {
        GetWindow<VRMClothingMergerWindow>("의상 합치기");
    }

    // 에디터 창 UI
    private void OnGUI()
    {
        EditorGUILayout.LabelField("VRM 의상 합치기", EditorStyles.boldLabel);

        using (new EditorGUILayout.VerticalScope(EditorStyles.helpBox))
        {
            m_avatarRoot = (GameObject)EditorGUILayout.ObjectField("아바타 루트", m_avatarRoot, typeof(GameObject), true);
            m_clothingRoot = (GameObject)EditorGUILayout.ObjectField("옷 루트", m_clothingRoot, typeof(GameObject), true);

            EditorGUILayout.Space(4);
            m_name = EditorGUILayout.TextField("이름", m_name);

            EditorGUILayout.Space(8);
            if (GUILayout.Button("실행", GUILayout.Height(26)))
            {
                Merge();
            }
        }

        if (m_avatarRoot == null)
        {
            EditorGUILayout.HelpBox("아바타 루트를 넣어주세요.", MessageType.Warning);
        }

        if (m_clothingRoot == null)
        {
            EditorGUILayout.HelpBox("옷 루트를 넣어주세요.", MessageType.Warning);
        }
    }

    // 실행: 본 이동 + SMR 이동 + 이름 뒤에 붙이기 + 옷 루트 삭제
    private void Merge()
    {
        if (EditorApplication.isPlayingOrWillChangePlaymode)
        {
            EditorUtility.DisplayDialog("의상 합치기", "Play Mode에서는 실행하지 마세요.", "OK");
            return;
        }

        if (m_avatarRoot == null || m_clothingRoot == null)
        {
            EditorUtility.DisplayDialog("의상 합치기", "아바타 루트와 옷 루트를 모두 넣어주세요.", "OK");
            return;
        }

        var avatarRootT = m_avatarRoot.transform;
        var clothingRootT = m_clothingRoot.transform;

        if (clothingRootT == avatarRootT)
        {
            EditorUtility.DisplayDialog("의상 합치기", "아바타 루트와 옷 루트가 같습니다.", "OK");
            return;
        }

        var avatarLookup = BuildFirstTransformByNameLookup(avatarRootT);
        var clothingSmrs = clothingRootT.GetComponentsInChildren<SkinnedMeshRenderer>(true)
            .Where(smr => smr != null)
            .ToArray();

        var undoGroup = Undo.GetCurrentGroup();
        Undo.IncrementCurrentGroup();
        Undo.SetCurrentGroupName("VRM 의상 합치기");

        var movedBones = 0;
        var movedSmrs = 0;
        var renamedSmrs = 0;
        var renamedBones = 0;
        var deletedObjects = 0;

        try
        {
            // 1) 본 합치기: 옷(SMR)이 실제로 참조하는 본들 중에서, 이름이 같은 아바타 본이 있으면 그 아래로 넣기
            const bool keepWorldPosition = true;

            var clothingBones = CollectClothingBonesFromSmrs(clothingSmrs, clothingRootT);
            foreach (var bone in clothingBones)
            {
                if (bone == null) continue;

                if (!avatarLookup.TryGetValue(bone.name, out var avatarMatch))
                {
                    continue;
                }

                if (avatarMatch == null) continue;
                if (avatarMatch == bone) continue;

                if (avatarMatch.IsChildOf(bone))
                {
                    continue;
                }

                if (bone.parent == avatarMatch)
                {
                    continue;
                }

                Undo.SetTransformParent(bone, avatarMatch, "Merge Clothing Bone");
                bone.SetParent(avatarMatch, keepWorldPosition);
                movedBones++;
            }

            // 2) 이름 붙이기: 옷에 일괄 적용
            var nameSuffix = NormalizeSuffix(m_name);
            if (!string.IsNullOrEmpty(nameSuffix))
            {
                foreach (var bone in clothingBones)
                {
                    if (bone == null) continue;
                    if (bone.name.EndsWith(nameSuffix, StringComparison.Ordinal)) continue;

                    Undo.RecordObject(bone.gameObject, "Rename Clothing Bone");
                    bone.name = bone.name + nameSuffix;
                    renamedBones++;
                }
            }

            // 3) SkinnedMeshRenderer 오브젝트를 아바타 루트 바로 아래로 옮기고, 이름도 동일하게 뒤에 붙이기
            foreach (var smr in clothingSmrs)
            {
                if (smr == null) continue;

                var t = smr.transform;
                if (t == null) continue;

                if (!(t.IsChildOf(avatarRootT) && t.parent == avatarRootT))
                {
                    Undo.SetTransformParent(t, avatarRootT, "Move Clothing SkinnedMesh");
                    t.SetParent(avatarRootT, keepWorldPosition);
                    movedSmrs++;
                }

                var suffix = NormalizeSuffix(m_name);
                if (!string.IsNullOrEmpty(suffix) && !t.name.EndsWith(suffix, StringComparison.Ordinal))
                {
                    Undo.RecordObject(t.gameObject, "Rename Clothing SkinnedMesh");
                    t.name = t.name + suffix;
                    renamedSmrs++;
                }
            }

            // 4) 옷 루트 삭제: 옷 루트 아래에 본이 남아있으면 SMR이 깨질 수 있으므로 삭제를 건너뜀
            {
                var stopAt = avatarRootT;

                var remainingUnderClothingRoot = clothingBones
                    .Where(b => b != null && b.IsChildOf(clothingRootT))
                    .Distinct()
                    .ToArray();

                if (remainingUnderClothingRoot.Length > 0)
                {
                    Debug.LogWarning($"[의상 합치기] 옷 루트 아래에 남은 본이 {remainingUnderClothingRoot.Length}개 있어서, 옷 루트 삭제를 건너뛰었습니다.");
                }
                else
                {
                    DeleteObjectAndEmptyParents(clothingRootT.gameObject, stopAt, deleteEmptyParents: true, ref deletedObjects);
                }
            }

            var summary = $"완료\n\n본 이동: {movedBones}\n본 이름변경: {renamedBones}\nSMR 이동: {movedSmrs}\nSMR 이름변경: {renamedSmrs}\n삭제: {deletedObjects}";
            EditorUtility.DisplayDialog("의상 합치기", summary, "OK");
        }
        catch (Exception ex)
        {
            Debug.LogException(ex);
            EditorUtility.DisplayDialog("의상 합치기", "실패했습니다. Console을 확인하세요.", "OK");
        }
        finally
        {
            Undo.CollapseUndoOperations(undoGroup);
        }
    }

    // 아바타 본 이름 -> Transform 룩업 만들기
    private Dictionary<string, Transform> BuildFirstTransformByNameLookup(Transform root)
    {
        var dict = new Dictionary<string, Transform>(StringComparer.Ordinal);
        foreach (var t in root.GetComponentsInChildren<Transform>(true))
        {
            if (t == null) continue;
            if (!dict.ContainsKey(t.name))
            {
                dict.Add(t.name, t);
            }
        }
        return dict;
    }

    // 옷 SMR이 실제로 쓰는 본(rootBone/bones) 수집
    private Transform[] CollectClothingBonesFromSmrs(SkinnedMeshRenderer[] clothingSmrs, Transform clothingRoot)
    {
        var set = new HashSet<Transform>();
        if (clothingSmrs == null) return Array.Empty<Transform>();

        foreach (var smr in clothingSmrs)
        {
            if (smr == null) continue;

            if (smr.rootBone != null) set.Add(smr.rootBone);
            var bones = smr.bones;
            if (bones != null)
            {
                for (var i = 0; i < bones.Length; i++)
                {
                    var b = bones[i];
                    if (b != null) set.Add(b);
                }
            }
        }

        if (clothingRoot != null)
        {
            set.RemoveWhere(t => t == null || !t.IsChildOf(clothingRoot));
        }

        return set.ToArray();
    }

    // 이름 뒤에 붙이는 값
    private string NormalizeSuffix(string raw)
    {
        if (string.IsNullOrWhiteSpace(raw))
        {
            return "";
        }

        var trimmed = raw.Trim();

        trimmed = trimmed.TrimEnd('_');
        if (string.IsNullOrEmpty(trimmed))
        {
            return "";
        }

        if (!trimmed.StartsWith("_", StringComparison.Ordinal))
        {
            trimmed = "_" + trimmed;
        }

        return trimmed;
    }

    // 오브젝트 삭제
    private void DeleteObjectAndEmptyParents(GameObject target, Transform stopAt, bool deleteEmptyParents, ref int deletedCount)
    {
        if (target == null) return;

        if (stopAt != null)
        {
            var t = target.transform;
            if (t == stopAt || stopAt.IsChildOf(t))
            {
                return;
            }
        }

        var parent = target.transform.parent;
        Undo.DestroyObjectImmediate(target);
        deletedCount++;

        if (!deleteEmptyParents)
        {
            return;
        }

        while (parent != null)
        {
            if (stopAt != null && parent == stopAt)
            {
                break;
            }

            if (parent.childCount > 0)
            {
                break;
            }

            var comps = parent.GetComponents<Component>();
            if (comps != null && comps.Length > 1)
            {
                break;
            }

            var go = parent.gameObject;
            parent = parent.parent;
            Undo.DestroyObjectImmediate(go);
            deletedCount++;
        }
    }

}
#endif
