# Security Vulnerability Audit - PyTerrainMap

**Audit Date:** 2026-10-27
**Repository:** /Users/georgimullassery/Desktop/Projects/PyTerrainMap
**Total Findings:** 652

## Summary
- **Critical:** 601
- **High:** 18
- **Medium:** 33

## Findings

### 1. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_code/code.py:164
- **Description:** eval()
- **Evidence (Line):** `def eval(self, code, **vars):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 2. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_code/code.py:173
- **Description:** eval()
- **Evidence (Line):** `return eval(code, self.f_globals, f_locals)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 3. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_code/code.py:326
- **Description:** eval()
- **Evidence (Line):** `# however via `exec(...)` / `eval(...)` they can be other types`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 4. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_code/code.py:326
- **Description:** exec()
- **Evidence (Line):** `# however via `exec(...)` / `eval(...)` they can be other types`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 5. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_py/path.py:1153
- **Description:** exec()
- **Evidence (Line):** `exec(f.read(), mod.__dict__)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 6. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/_py/path.py:1159
- **Description:** exec()
- **Evidence (Line):** `def sysexec(self, *argv: os.PathLike[str], **popen_opts: Any) -> str:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 7. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/assertion/rewrite.py:188
- **Description:** exec()
- **Evidence (Line):** `exec(co, module.__dict__)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 8. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/assertion/rewrite.py:393
- **Description:** marshal load
- **Evidence (Line):** `co = marshal.load(fp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 9. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/capture.py:1112
- **Description:** os.system()
- **Evidence (Line):** `os.system('echo "hello"')`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 10. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/capture.py:1140
- **Description:** os.system()
- **Evidence (Line):** `os.system('echo "hello"')`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 11. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/mark/__init__.py:66
- **Description:** eval()
- **Evidence (Line):** `def test_eval(test_input, expected):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 12. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/mark/__init__.py:67
- **Description:** eval()
- **Evidence (Line):** `assert eval(test_input) == expected`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 13. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/mark/expression.py:295
- **Description:** eval()
- **Evidence (Line):** `"""Adapts a matcher function to a locals mapping as required by eval()."""`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 14. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/mark/expression.py:353
- **Description:** eval()
- **Evidence (Line):** `return bool(eval(self._code, {"__builtins__": {}}, MatcherAdapter(matcher)))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 15. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/pytester.py:296
- **Description:** eval()
- **Evidence (Line):** `if eval(check, backlocals, call.__dict__):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 16. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/skipping.py:92
- **Description:** eval()
- **Evidence (Line):** `If an old-style string condition is given, it is eval()'d, otherwise the`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 17. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/_pytest/skipping.py:119
- **Description:** eval()
- **Evidence (Line):** `result = eval(condition_code, globals_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 18. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/anyio/_backends/_asyncio.py:2796
- **Description:** exec()
- **Evidence (Line):** `# Use loop.subprocess_shell()/subprocess_exec() rather than their`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 19. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/anyio/_backends/_asyncio.py:2810
- **Description:** exec()
- **Evidence (Line):** `transport, protocol = await loop.subprocess_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 20. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/anyio/to_interpreter.py:123
- **Description:** exec()
- **Evidence (Line):** `exc_info = _interpreters.exec(self._interpreter_id, _run_func)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 21. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/anyio/to_interpreter.py:130
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(res)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 22. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/anyio/to_process.py:95
- **Description:** pickle load/dumps
- **Evidence (Line):** `retval = pickle.loads(pickled_response)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 23. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/anyio/to_process.py:220
- **Description:** pickle load/dumps
- **Evidence (Line):** `command, *args = pickle.load(stdin.buffer)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 24. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/coverage/execfile.py:222
- **Description:** exec()
- **Evidence (Line):** `exec(code, main_mod.__dict__)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 25. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/coverage/execfile.py:335
- **Description:** marshal load
- **Evidence (Line):** `code = marshal.load(fpyc)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 26. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/coverage/htmlfiles/coverage_html.js:236
- **Description:** exec()
- **Evidence (Line):** `const match = /\.([0-9]+)/.exec(cell.textContent);`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 27. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/coverage/parser.py:691
- **Description:** eval()
- **Evidence (Line):** `return True, eval(node.id)  # pylint: disable=eval-used`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 28. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/coverage/templite.py:70
- **Description:** exec()
- **Evidence (Line):** `exec(python_source, global_namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 29. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/cv2/load_config_py3.py:9
- **Description:** exec()
- **Evidence (Line):** `exec(code, g_vars, l_vars)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 30. [Critical] Hardcoded Secret
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_urls.py:336
- **Description:** Password Assignment detected
- **Evidence (Line):** `username="jo@gmail.com", password="a secret"`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 31. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/_store_backends.py:196
- **Description:** pickle load/dumps
- **Evidence (Line):** `item = numpy_pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 32. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/_store_backends.py:198
- **Description:** pickle load/dumps
- **Evidence (Line):** `item = numpy_pickle.load(filename, mmap_mode=mmap_mode)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 33. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/externals/loky/backend/fork_exec.py:12
- **Description:** exec()
- **Evidence (Line):** `def fork_exec(cmd, keep_fds, env=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 34. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/externals/loky/backend/fork_exec.py:46
- **Description:** exec()
- **Evidence (Line):** `return _posixsubprocess.fork_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 35. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/externals/loky/backend/popen_loky_posix.py:128
- **Description:** exec()
- **Evidence (Line):** `pid = fork_exec(cmd_python, self._fds, env=process_obj.env)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 36. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/externals/loky/backend/popen_loky_posix.py:174
- **Description:** pickle load/dumps
- **Evidence (Line):** `prep_data = pickle.load(from_parent)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 37. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/externals/loky/backend/popen_loky_posix.py:176
- **Description:** pickle load/dumps
- **Evidence (Line):** `process_obj = pickle.load(from_parent)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 38. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/hashing.py:231
- **Description:** pickle load/dumps
- **Evidence (Line):** `# ``pickle.loads(pickle.dumps(np.dtype('f4'))) is not`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 39. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/hashing.py:236
- **Description:** pickle load/dumps
- **Evidence (Line):** `# and ``[np.dtype('f4'), pickle.loads(pickle.dumps('f4'))]``.`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 40. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/memory.py:178
- **Description:** eval()
- **Evidence (Line):** `(namely eval(repr(memorized_instance)) works).`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 41. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/memory.py:1192
- **Description:** eval()
- **Evidence (Line):** `def eval(self, func, *args, **kwargs):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 42. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/parallel.py:1771
- **Description:** eval()
- **Evidence (Line):** `def _wait_retrieval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 43. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/parallel.py:1800
- **Description:** eval()
- **Evidence (Line):** `while self._wait_retrieval():`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 44. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_func_inspect.py:366
- **Description:** exec()
- **Evidence (Line):** `exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 45. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:350
- **Description:** pickle load/dumps
- **Evidence (Line):** `c = pickle.loads(pickle.dumps(b))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 46. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:370
- **Description:** pickle load/dumps
- **Evidence (Line):** `dt1_roundtripped = pickle.loads(pickle.dumps(dt1))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 47. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:383
- **Description:** pickle load/dumps
- **Evidence (Line):** `complex_dt1_roundtripped = pickle.loads(pickle.dumps(complex_dt1))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 48. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:129
- **Description:** eval()
- **Evidence (Line):** `assert memory.eval(f, 1) == out`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 49. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:400
- **Description:** eval()
- **Evidence (Line):** `a1 = eval("lambda x: x")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 50. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:402
- **Description:** eval()
- **Evidence (Line):** `b1 = eval("lambda x: x+1")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 51. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:430
- **Description:** eval()
- **Evidence (Line):** `def test_memory_eval(tmpdir):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 52. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:434
- **Description:** eval()
- **Evidence (Line):** `m = eval("lambda x: x")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 53. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:166
- **Description:** exec()
- **Evidence (Line):** `exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 54. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:257
- **Description:** exec()
- **Evidence (Line):** `exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 55. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:686
- **Description:** pickle load/dumps
- **Evidence (Line):** `h = pickle.loads(pickle.dumps(g))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 56. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:692
- **Description:** pickle load/dumps
- **Evidence (Line):** `memory2 = pickle.loads(pickle.dumps(memory))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 57. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:697
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(pickle.dumps(memory))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 58. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:699
- **Description:** pickle load/dumps
- **Evidence (Line):** `gp = pickle.loads(pickle.dumps(g))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 59. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:792
- **Description:** pickle load/dumps
- **Evidence (Line):** `result2 = pickle.load(fp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 60. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:1425
- **Description:** pickle load/dumps
- **Evidence (Line):** `memorized_result_loads = pickle.loads(memorized_result_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 61. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:1459
- **Description:** pickle load/dumps
- **Evidence (Line):** `memory_reloaded = pickle.loads(pickle.dumps(memory))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 62. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:1472
- **Description:** pickle load/dumps
- **Evidence (Line):** `func_cached_reloaded = pickle.loads(pickle.dumps(func_cached))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 63. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory.py:1485
- **Description:** pickle load/dumps
- **Evidence (Line):** `memorized_result_reloaded = pickle.loads(pickle.dumps(memorized_result))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 64. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_memory_async.py:69
- **Description:** eval()
- **Evidence (Line):** `evaled = await memory.eval(f, 1)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 65. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:119
- **Description:** pickle load/dumps
- **Evidence (Line):** `_member = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 66. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:159
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_ = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 67. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:172
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_ = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 68. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:185
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_loaded = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 69. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:197
- **Description:** pickle load/dumps
- **Evidence (Line):** `arr_reloaded = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 70. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:208
- **Description:** pickle load/dumps
- **Evidence (Line):** `b = numpy_pickle.load(filename, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 71. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:216
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_loaded = numpy_pickle.load(filename, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 72. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:229
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_loaded = numpy_pickle.load(filename, mmap_mode="r+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 73. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:235
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(filename, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 74. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:240
- **Description:** pickle load/dumps
- **Evidence (Line):** `numpy_pickle.load(filename, mmap_mode="w+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 75. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:257
- **Description:** pickle load/dumps
- **Evidence (Line):** `a_clone, b_clone = numpy_pickle.load(filename, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 76. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:312
- **Description:** pickle load/dumps
- **Evidence (Line):** `loaded = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 77. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:326
- **Description:** pickle load/dumps
- **Evidence (Line):** `b = numpy_pickle.load(filename, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 78. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:338
- **Description:** pickle load/dumps
- **Evidence (Line):** `reloaded_obj = numpy_pickle.load(this_filename, mmap_mode="r+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 79. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:392
- **Description:** pickle load/dumps
- **Evidence (Line):** `result_list = numpy_pickle.load(fname)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 80. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:415
- **Description:** pickle load/dumps
- **Evidence (Line):** `le_array_native_load, be_array_native_load = numpy_pickle.load(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 81. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:422
- **Description:** pickle load/dumps
- **Evidence (Line):** `le_array_nonnative_load, be_array_nonnative_load = numpy_pickle.load(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 82. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:437
- **Description:** pickle load/dumps
- **Evidence (Line):** `numpy_pickle.load(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 83. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:465
- **Description:** pickle load/dumps
- **Evidence (Line):** `result_list = numpy_pickle.load(filename, mmap_mode=mmap_mode)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 84. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:529
- **Description:** pickle load/dumps
- **Evidence (Line):** `numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 85. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:719
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(dump_filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 86. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:763
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(filename_raw)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 87. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:794
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(dump_fname)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 88. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:815
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 89. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:820
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded_2 = numpy_pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 90. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:836
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 91. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:852
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj_reloaded = numpy_pickle.load(f, mmap_mode="r+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 92. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:867
- **Description:** pickle load/dumps
- **Evidence (Line):** `numpy_pickle.load(f, mmap_mode="r+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 93. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:884
- **Description:** pickle load/dumps
- **Evidence (Line):** `numpy_pickle.load(buf, mmap_mode="r+")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 94. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1008
- **Description:** pickle load/dumps
- **Evidence (Line):** `c = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 95. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1017
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert numpy_pickle.load(filename) == value`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 96. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1019
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert numpy_pickle.load(Path(filename)) == value`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 97. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1036
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert numpy_pickle.load(path) == value`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 98. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1053
- **Description:** pickle load/dumps
- **Evidence (Line):** `array_reloaded = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 99. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1067
- **Description:** pickle load/dumps
- **Evidence (Line):** `array_reloaded = numpy_pickle.load(filename)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 100. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1088
- **Description:** pickle load/dumps
- **Evidence (Line):** `array_reloaded = numpy_pickle.load(cf)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 101. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1099
- **Description:** pickle load/dumps
- **Evidence (Line):** `array_reloaded = numpy_pickle.load(cf)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 102. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1114
- **Description:** pickle load/dumps
- **Evidence (Line):** `memmaps = numpy_pickle.load(fname, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 103. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1217
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert numpy_pickle.load(fname) == data`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 104. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1223
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert numpy_pickle.load(fname) == data`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 105. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1254
- **Description:** pickle load/dumps
- **Evidence (Line):** `memmap = numpy_pickle.load(fname, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 106. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1270
- **Description:** pickle load/dumps
- **Evidence (Line):** `l_reloaded = numpy_pickle.load(fname, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 107. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_numpy_pickle.py:1293
- **Description:** pickle load/dumps
- **Evidence (Line):** `d_reloaded = numpy_pickle.load(fname, mmap_mode="r")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 108. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_store_backends.py:33
- **Description:** pickle load/dumps
- **Evidence (Line):** `reloaded = cpickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 109. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_utils.py:9
- **Description:** exec()
- **Evidence (Line):** `"exec('import os')",`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 110. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/narwhals/_polars/expr.py:455
- **Description:** eval()
- **Evidence (Line):** `.list.eval(pl.element().str.to_titlecase())`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 111. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/_internal.py:204
- **Description:** eval()
- **Evidence (Line):** `newitem = (dtype, ast.literal_eval(repeats))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 112. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/arrayprint.py:1577
- **Description:** eval()
- **Evidence (Line):** `>>> assert eval(dtype_short_repr(dt)) == dt`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 113. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/records.py:703
- **Description:** pickle load/dumps
- **Evidence (Line):** `>>> pickle.loads(pickle.dumps(r))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 114. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test__exceptions.py:19
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(pickle.dumps(error))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 115. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test__exceptions.py:84
- **Description:** pickle load/dumps
- **Evidence (Line):** `exc2 = pickle.loads(pickle.dumps(exc))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 116. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_arrayprint.py:343
- **Description:** eval()
- **Evidence (Line):** `assert_equal(eval(repr(a), vars(np)), a)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 117. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_arrayprint.py:344
- **Description:** eval()
- **Evidence (Line):** `assert_equal(eval(repr(a[0]), {'np': np}), a[0])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 118. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_custom_dtypes.py:504
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 119. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:912
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pickle.dumps(dt, protocol=proto)), dt)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 120. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:914
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pickle.dumps(dt, protocol=proto)), dt)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 121. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:916
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pickle.dumps(scalar, protocol=proto)),`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 122. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:919
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pickle.dumps(delta, protocol=proto)),`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 123. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:926
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pkl), np.dtype('<M8[7D]'))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 124. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:930
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pkl), np.dtype('<M8[W]'))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 125. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:934
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(pkl), np.dtype('>M8[us]'))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 126. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_datetime.py:941
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(pickle.dumps(dt, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 127. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_dtype.py:996
- **Description:** eval()
- **Evidence (Line):** `assert_equal(np.dtype(eval(str(dt))), dt)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 128. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_dtype.py:1316
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickled = pickle.loads(buf)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 129. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_dtype.py:1387
- **Description:** pickle load/dumps
- **Evidence (Line):** `roundtrip_DType = pickle.loads(pickle.dumps(DType, proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 130. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_dtype.py:1398
- **Description:** pickle load/dumps
- **Evidence (Line):** `roundtrip_dt = pickle.loads(pickle.dumps(dt, proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 131. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:1704
- **Description:** eval()
- **Evidence (Line):** `assert_array_equal(eval(repr(xx), {"np": np, "array": np.array}), xx)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 132. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:4455
- **Description:** eval()
- **Evidence (Line):** `MyType = eval("type('MyType', (base,), class_namespace)",`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 133. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:11606
- **Description:** exec()
- **Evidence (Line):** `interp.exec("import numpy")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 134. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:194
- **Description:** pickle load/dumps
- **Evidence (Line):** `vals = pickle.loads(pickle.dumps(a, v))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 135. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:1856
- **Description:** pickle load/dumps
- **Evidence (Line):** `scalar = pickle.loads(pickle.dumps(scalar))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 136. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:2008
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(zs, pickle.loads(zs.dumps()))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 137. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:2015
- **Description:** pickle load/dumps
- **Evidence (Line):** `zs2 = pickle.loads(p)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 138. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:2024
- **Description:** pickle load/dumps
- **Evidence (Line):** `pk_load = pickle.loads(pk_dmp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 139. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:2035
- **Description:** pickle load/dumps
- **Evidence (Line):** `array_from_buffer = pickle.loads(bytes_string, buffers=buffers)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 140. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:4953
- **Description:** pickle load/dumps
- **Evidence (Line):** `depickled_arr_with_object = pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 141. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:4955
- **Description:** pickle load/dumps
- **Evidence (Line):** `depickled_arr_without_object = pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 142. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:4978
- **Description:** pickle load/dumps
- **Evidence (Line):** `depickled_f_contiguous_array = pickle.loads(bytes_string,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 143. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5009
- **Description:** pickle load/dumps
- **Evidence (Line):** `depickled_transposed_contiguous_array = pickle.loads(bytes_string,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 144. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5025
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(x, pickle.loads(c_contiguous_dumped))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 145. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5027
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(x, pickle.loads(f_contiguous_dumped))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 146. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5029
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(x, pickle.loads(transposed_contiguous_dumped))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 147. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5031
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(x, pickle.loads(no_contiguous_dumped))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 148. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5043
- **Description:** pickle load/dumps
- **Evidence (Line):** `depickled_non_contiguous_array = pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 149. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5068
- **Description:** pickle load/dumps
- **Evidence (Line):** `a, pickle.loads(pickle.dumps(a, protocol=proto)),`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 150. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5077
- **Description:** pickle load/dumps
- **Evidence (Line):** `return pickle.loads(obj, encoding='latin1')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 151. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:5178
- **Description:** pickle load/dumps
- **Evidence (Line):** `new = pickle.loads(pickle.dumps(original_byte_reversed))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 152. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_multiarray.py:9146
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(pickle_obj)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 153. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_overrides.py:221
- **Description:** pickle load/dumps
- **Evidence (Line):** `roundtripped = pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 154. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:184
- **Description:** eval()
- **Evidence (Line):** `recordarr_r = eval("np." + repr(recordarr), {'np': np})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 155. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:185
- **Description:** eval()
- **Evidence (Line):** `recarr_r = eval("np." + repr(recarr), {'np': np})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 156. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:187
- **Description:** eval()
- **Evidence (Line):** `recordview_r = eval("np." + repr(recordview), {'np': np, 'numpy': np})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 157. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:430
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(a, pickle.loads(pickle.dumps(a, protocol=proto)))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 158. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:431
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(a[0], pickle.loads(pickle.dumps(a[0],`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 159. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:437
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(a, pickle.loads(pickle.dumps(a, protocol=proto)))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 160. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:438
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(a[0], pickle.loads(pickle.dumps(a[0],`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 161. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:445
- **Description:** pickle load/dumps
- **Evidence (Line):** `pa = pickle.loads(pickle.dumps(a[0], protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 162. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_records.py:469
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(dump)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 163. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:57
- **Description:** pickle load/dumps
- **Evidence (Line):** `b = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 164. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:369
- **Description:** pickle load/dumps
- **Evidence (Line):** `dt_ = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 165. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:495
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.loads(data, encoding='bytes')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 166. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:840
- **Description:** pickle load/dumps
- **Evidence (Line):** `new = pickle.loads(pickle.dumps(el, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 167. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1076
- **Description:** pickle load/dumps
- **Evidence (Line):** `y = pickle.loads(pickle.dumps(x, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 168. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1092
- **Description:** pickle load/dumps
- **Evidence (Line):** `xp = pickle.load(f, encoding='latin1')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 169. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1291
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_(pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 170. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1293
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_(pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 171. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1899
- **Description:** pickle load/dumps
- **Evidence (Line):** `data = pickle.loads(blob)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 172. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1911
- **Description:** pickle load/dumps
- **Evidence (Line):** `data = pickle.loads(pickle.dumps(data, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 173. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1932
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.loads(data, encoding='latin1')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 174. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1961
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.loads(data, encoding='latin1')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 175. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:1970
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.loads(data, encoding='koi8-r')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 176. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:2215
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 177. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:2435
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_equal(pickle.loads(dumped), arr)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 178. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_regression.py:2569
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.loads(test_data, encoding='bytes')`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 179. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_scalarmath.py:610
- **Description:** eval()
- **Evidence (Line):** `#            val2 = eval(val_repr)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 180. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_scalarmath.py:636
- **Description:** eval()
- **Evidence (Line):** `val2 = t(eval(val_repr))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 181. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:244
- **Description:** eval()
- **Evidence (Line):** `func = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 182. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:512
- **Description:** eval()
- **Evidence (Line):** `func = eval(intrin[:3])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 183. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:642
- **Description:** eval()
- **Evidence (Line):** `func = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 184. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:703
- **Description:** eval()
- **Evidence (Line):** `npyv_load_tillz, npyv_load_till = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 185. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:723
- **Description:** eval()
- **Evidence (Line):** `npyv_store_till = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 186. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:743
- **Description:** eval()
- **Evidence (Line):** `npyv_loadn = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 187. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:769
- **Description:** eval()
- **Evidence (Line):** `npyv_loadn_tillz, npyv_loadn_till = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 188. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:806
- **Description:** eval()
- **Evidence (Line):** `npyv_storen = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 189. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:845
- **Description:** eval()
- **Evidence (Line):** `npyv_storen_till = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 190. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:897
- **Description:** eval()
- **Evidence (Line):** `intrin = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 191. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_simd.py:1104
- **Description:** eval()
- **Evidence (Line):** `func = eval(intrin)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 192. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_stringdtype.py:659
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 193. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_ufunc.py:203
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_(pickle.loads(pickle.dumps(np.sin,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 194. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_ufunc.py:208
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.loads(pickle.dumps(_rational_tests.test_add,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 195. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_ufunc.py:215
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_(pickle.loads(astring) is np.cos)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 196. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_ufunc.py:224
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj = pickle.loads(pickle.dumps(_pickleable_module_global.ufunc))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 197. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_ufunc.py:499
- **Description:** pickle load/dumps
- **Evidence (Line):** `int64_2 = pickle.loads(pickle.dumps(int64))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 198. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_umath.py:513
- **Description:** eval()
- **Evidence (Line):** `a, b, divisors = eval(ex_val)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 199. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_umath.py:576
- **Description:** eval()
- **Evidence (Line):** `a = eval(ex_val)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 200. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_umath_accuracy.py:75
- **Description:** eval()
- **Evidence (Line):** `dtype=eval(datatype))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 201. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/_core/tests/test_umath_accuracy.py:78
- **Description:** eval()
- **Evidence (Line):** `dtype=eval(datatype))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 202. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/auxfuncs.py:632
- **Description:** eval()
- **Evidence (Line):** `return eval(f"{l1}:{' and '.join(l2)}")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 203. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/auxfuncs.py:640
- **Description:** eval()
- **Evidence (Line):** `return eval(f"{l1}:{' or '.join(l2)}")`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 204. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/auxfuncs.py:644
- **Description:** eval()
- **Evidence (Line):** `return eval('lambda v,f=f:not f(v)')`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 205. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/capi_maps.py:159
- **Description:** eval()
- **Evidence (Line):** `d = eval(f.read().lower(), {}, {})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 206. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/capi_maps.py:299
- **Description:** eval()
- **Evidence (Line):** `ret['size'] = repr(eval(ret['size']))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 207. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/capi_maps.py:447
- **Description:** eval()
- **Evidence (Line):** `v = eval(v, {}, {})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 208. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:1325
- **Description:** eval()
- **Evidence (Line):** `v = eval(initexpr, {}, params)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 209. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2272
- **Description:** eval()
- **Evidence (Line):** `def myeval(e, g=None, l=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 210. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2274
- **Description:** eval()
- **Evidence (Line):** `r = eval(e, g, l)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 211. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2305
- **Description:** eval()
- **Evidence (Line):** `c = int(myeval(e, {}, {}))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 212. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2326
- **Description:** eval()
- **Evidence (Line):** `b = myeval(ee, {}, {})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 213. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2331
- **Description:** eval()
- **Evidence (Line):** `a = myeval(ee, {}, {}) - b`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 214. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2336
- **Description:** eval()
- **Evidence (Line):** `c = myeval(ee, {}, {})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 215. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2342
- **Description:** eval()
- **Evidence (Line):** `c2 = myeval(ee, {}, {})`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 216. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2532
- **Description:** eval()
- **Evidence (Line):** `params[n] = param_eval(v, g_params, params, dimspec=dimspec)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 217. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2562
- **Description:** eval()
- **Evidence (Line):** `value = eval(value, {}, params)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 218. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2639
- **Description:** eval()
- **Evidence (Line):** `l = str(eval(l, {}, params))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 219. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2648
- **Description:** eval()
- **Evidence (Line):** `l = str(eval(l, {}, params))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 220. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2916
- **Description:** eval()
- **Evidence (Line):** `kindselect['kind'] = eval(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 221. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2966
- **Description:** eval()
- **Evidence (Line):** `def param_eval(v, g_params, params, dimspec=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 222. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:2987
- **Description:** eval()
- **Evidence (Line):** `p = eval(v, g_params, params)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 223. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:3018
- **Description:** eval()
- **Evidence (Line):** `item = eval(item, g_params, params)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 224. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/crackfortran.py:3470
- **Description:** eval()
- **Evidence (Line):** `v = eval(v)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 225. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/tests/test_crackfortran.py:378
- **Description:** eval()
- **Evidence (Line):** `ret = crackfortran.param_eval(v, g_params, params, dimspec=dimspec)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 226. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/tests/test_crackfortran.py:388
- **Description:** eval()
- **Evidence (Line):** `ret = crackfortran.param_eval(v, g_params, params, dimspec=dimspec)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 227. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/tests/test_crackfortran.py:407
- **Description:** eval()
- **Evidence (Line):** `ret = crackfortran.param_eval(v, g_params, params, dimspec=None)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 228. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/tests/test_f2py2e.py:889
- **Description:** exec()
- **Evidence (Line):** `def test_npd_f77exec():`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 229. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/f2py/tests/test_f2py2e.py:897
- **Description:** exec()
- **Evidence (Line):** `def test_npd_f90exec():`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 230. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:536
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 231. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:563
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 232. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:658
- **Description:** eval()
- **Evidence (Line):** `d = ast.literal_eval(header)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 233. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:663
- **Description:** eval()
- **Evidence (Line):** `d = ast.literal_eval(header)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 234. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:799
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 235. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:929
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 236. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_format_impl.py:838
- **Description:** pickle load/dumps
- **Evidence (Line):** `array = pickle.load(fp, **pickle_kwargs)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 237. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_npyio_impl.py:151
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 238. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_npyio_impl.py:356
- **Description:** eval()
- **Evidence (Line):** `See :py:func:`ast.literal_eval()` for details.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 239. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_npyio_impl.py:492
- **Description:** pickle load/dumps
- **Evidence (Line):** `"`allow_pickle=` keyword argument or `pickle.load()`.")`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 240. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/lib/_npyio_impl.py:494
- **Description:** pickle load/dumps
- **Evidence (Line):** `return pickle.load(fid, **pickle_kwargs)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 241. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:733
- **Description:** pickle load/dumps
- **Evidence (Line):** `a_pickled = pickle.loads(pickle.dumps(a, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 242. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:748
- **Description:** pickle load/dumps
- **Evidence (Line):** `a_pickled = pickle.loads(pickle.dumps(a, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 243. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:757
- **Description:** pickle load/dumps
- **Evidence (Line):** `mc_pickled = pickle.loads(pickle.dumps(mc, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 244. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:767
- **Description:** pickle load/dumps
- **Evidence (Line):** `a_pickled = pickle.loads(pickle.dumps(a, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 245. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:776
- **Description:** pickle load/dumps
- **Evidence (Line):** `test = pickle.loads(pickle.dumps(b, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 246. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_core.py:5738
- **Description:** pickle load/dumps
- **Evidence (Line):** `res = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 247. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_mrecords.py:291
- **Description:** pickle load/dumps
- **Evidence (Line):** `mrec_ = pickle.loads(_)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 248. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/ma/tests/test_old_ma.py:617
- **Description:** pickle load/dumps
- **Evidence (Line):** `y = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 249. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/matrixlib/tests/test_masked_matrix.py:89
- **Description:** pickle load/dumps
- **Evidence (Line):** `a_pickled = pickle.loads(pickle.dumps(a, protocol=proto))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 250. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:245
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval(0,hermeline(3, 2))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 251. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:247
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval(1,hermeline(3, 2))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 252. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:303
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval((-1, 0, 1), coef)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 253. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:306
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval((-1j, 1j), coef)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 254. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:790
- **Description:** eval()
- **Evidence (Line):** `tmp[0] += k[i] - hermeval(lbnd, tmp)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 255. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:796
- **Description:** eval()
- **Evidence (Line):** `def hermeval(x, c, tensor=True):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 256. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:856
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval(1, coef)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 257. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:858
- **Description:** eval()
- **Evidence (Line):** `>>> hermeval([[1,2],[3,4]], coef)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 258. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:1138
- **Description:** eval()
- **Evidence (Line):** ```hermeval(x, c)`` are the same up to roundoff. This equivalence is`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 259. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/hermite_e.py:1407
- **Description:** eval()
- **Evidence (Line):** `>>> y = hermeval(x, [1, 2, 3]) + err`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 260. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:79
- **Description:** eval()
- **Evidence (Line):** `val1 = herme.hermeval(self.x, pol1)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 261. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:83
- **Description:** eval()
- **Evidence (Line):** `val2 = herme.hermeval(self.x, pol2)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 262. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:85
- **Description:** eval()
- **Evidence (Line):** `val3 = herme.hermeval(self.x, pol3)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 263. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:120
- **Description:** eval()
- **Evidence (Line):** `def test_hermeval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 264. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:122
- **Description:** eval()
- **Evidence (Line):** `assert_equal(herme.hermeval([], [1]).size, 0)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 265. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:130
- **Description:** eval()
- **Evidence (Line):** `res = herme.hermeval(x, [0] * i + [1])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 266. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:137
- **Description:** eval()
- **Evidence (Line):** `assert_equal(herme.hermeval(x, [1]).shape, dims)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 267. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:138
- **Description:** eval()
- **Evidence (Line):** `assert_equal(herme.hermeval(x, [1, 0]).shape, dims)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 268. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:139
- **Description:** eval()
- **Evidence (Line):** `assert_equal(herme.hermeval(x, [1, 0, 0]).shape, dims)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 269. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:258
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(-1, hermeint), i)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 270. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:379
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(v[..., i], herme.hermeval(x, coef))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 271. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:387
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(v[..., i], herme.hermeval(x, coef))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 272. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:444
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef3), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 273. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:447
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef3), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 274. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:451
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef4), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 275. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:454
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef4), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 276. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:458
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef4), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 277. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:487
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef1), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 278. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:489
- **Description:** eval()
- **Evidence (Line):** `assert_almost_equal(herme.hermeval(x, coef2), y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 279. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_hermite_e.py:535
- **Description:** eval()
- **Evidence (Line):** `res = herme.hermeval(roots, pol)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 280. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/polynomial/tests/test_polynomial.py:62
- **Description:** pickle load/dumps
- **Evidence (Line):** `y = pickle.loads(pickle.dumps(x))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 281. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_direct.py:328
- **Description:** pickle load/dumps
- **Evidence (Line):** `reloaded = pickle.loads(bitgen_pkl)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 282. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_direct.py:336
- **Description:** pickle load/dumps
- **Evidence (Line):** `aa = pickle.loads(pickle.dumps(ss))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 283. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_direct.py:346
- **Description:** pickle load/dumps
- **Evidence (Line):** `bg_plk = pickle.loads(pickle.dumps(bit_generator))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 284. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_direct.py:352
- **Description:** pickle load/dumps
- **Evidence (Line):** `bg_plk = pickle.loads(pickle.dumps(bit_generator))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 285. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_direct.py:580
- **Description:** pickle load/dumps
- **Evidence (Line):** `sfc = pickle.load(gz)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 286. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_generator_mt19937.py:2802
- **Description:** pickle load/dumps
- **Evidence (Line):** `rg_plk = pickle.loads(pickle.dumps(rg))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 287. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_generator_mt19937.py:2808
- **Description:** pickle load/dumps
- **Evidence (Line):** `rg_plk = pickle.loads(pickle.dumps(rg))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 288. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_generator_mt19937.py:2824
- **Description:** pickle load/dumps
- **Evidence (Line):** `rg = pickle.load(gz)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 289. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_randomstate.py:272
- **Description:** pickle load/dumps
- **Evidence (Line):** `rs_unpick = pickle.loads(pickle.dumps(random_state))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 290. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_smoke.py:488
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpick = pickle.loads(pick)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 291. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/random/tests/test_smoke.py:494
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpick = pickle.loads(pick)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 292. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/testing/_private/utils.py:1352
- **Description:** exec()
- **Evidence (Line):** `exec(astr, dict)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 293. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/testing/_private/utils.py:1640
- **Description:** exec()
- **Evidence (Line):** `exec(code, globs, locs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 294. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/numpy/tests/test_public_api.py:322
- **Description:** eval()
- **Evidence (Line):** `eval(module_name)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 295. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/numpy/tests/test_reloading.py:40
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_(pickle.loads(pickle.dumps(np._NoValue,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 296. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/packaging/_parser.py:397
- **Description:** eval()
- **Evidence (Line):** `value = ast.literal_eval(python_str)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 297. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/packaging/_structures.py:10
- **Description:** pickle load/dumps
- **Evidence (Line):** `stand-in classes so that ``pickle.loads()`` can resolve those references.`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 298. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_internal/commands/configuration.py:248
- **Description:** subprocess with shell=True
- **Evidence (Line):** `subprocess.check_call(f'{editor} "{fname}"', shell=True)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 299. [Critical] Hardcoded Secret
- **File:** .venv-test/lib/python3.12/site-packages/pip/_internal/network/auth.py:486
- **Description:** Password Assignment detected
- **Evidence (Line):** `password = ask_password("Password: ")`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 300. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/distlib/scripts.py:165
- **Description:** exec()
- **Evidence (Line):** `# shebang, or else using os.exec() to run the entry script will`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 301. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/packaging/_parser.py:372
- **Description:** eval()
- **Evidence (Line):** `value = ast.literal_eval(python_str)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 302. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/packaging/_structures.py:10
- **Description:** pickle load/dumps
- **Evidence (Line):** `stand-in classes so that ``pickle.loads()`` can resolve those references.`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 303. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/pkg_resources/__init__.py:1714
- **Description:** exec()
- **Evidence (Line):** `exec(code, namespace, namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 304. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/pkg_resources/__init__.py:1725
- **Description:** exec()
- **Evidence (Line):** `exec(script_code, namespace, namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 305. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/pygments/formatters/__init__.py:91
- **Description:** eval()
- **Evidence (Line):** `this method is equivalent to running ``eval()`` on the input file. The formatter is`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 306. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/pygments/formatters/__init__.py:103
- **Description:** exec()
- **Evidence (Line):** `exec(f.read(), custom_namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 307. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/pygments/lexers/__init__.py:154
- **Description:** exec()
- **Evidence (Line):** `exec(f.read(), custom_namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 308. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/rich/markup.py:190
- **Description:** eval()
- **Evidence (Line):** `meta_params = literal_eval(parameters)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 309. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_hooks.py:512
- **Description:** exec()
- **Evidence (Line):** `return self._hookexec(self.name, self._hookimpls.copy(), kwargs, firstresult)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 310. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_hooks.py:534
- **Description:** exec()
- **Evidence (Line):** `res = self._hookexec(self.name, self._hookimpls.copy(), kwargs, False)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 311. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_hooks.py:573
- **Description:** exec()
- **Evidence (Line):** `return self._hookexec(self.name, hookimpls, kwargs, firstresult)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 312. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_hooks.py:580
- **Description:** exec()
- **Evidence (Line):** `res = self._hookexec(self.name, [method], kwargs, False)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 313. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_manager.py:111
- **Description:** exec()
- **Evidence (Line):** `def _hookexec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 314. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_manager.py:120
- **Description:** exec()
- **Evidence (Line):** `return self._inner_hookexec(hook_name, methods, kwargs, firstresult)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 315. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pluggy/_manager.py:464
- **Description:** exec()
- **Evidence (Line):** `def traced_hookexec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 316. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:599
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(validator)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 317. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:637
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(serializer)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 318. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:675
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(serializer)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 319. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:707
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(serializer)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 320. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:755
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(unwrap_wrapped_function(function))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 321. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_decorators.py:779
- **Description:** eval()
- **Evidence (Line):** `sig = signature_no_eval(unwrap_wrapped_function(function))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 322. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_fields.py:849
- **Description:** eval()
- **Evidence (Line):** `sig = _typing_extra.signature_no_eval(default_factory)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 323. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_generate_schema.py:2030
- **Description:** eval()
- **Evidence (Line):** `sig = _typing_extra.signature_no_eval(function)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 324. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_generate_schema.py:2055
- **Description:** eval()
- **Evidence (Line):** `sig = _typing_extra.signature_no_eval(function)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 325. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_generate_schema.py:2134
- **Description:** eval()
- **Evidence (Line):** `sig = _typing_extra.signature_no_eval(function)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 326. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_namespace_utils.py:38
- **Description:** eval()
- **Evidence (Line):** `return eval(typ, *ns)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 327. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_signature.py:96
- **Description:** eval()
- **Evidence (Line):** `present_params = signature_no_eval(init).parameters.values()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 328. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_typing_extra.py:458
- **Description:** eval()
- **Evidence (Line):** `def signature_no_eval(f: Callable[..., Any]) -> Signature:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 329. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/_internal/_validate_call.py:107
- **Description:** eval()
- **Evidence (Line):** `signature = signature_no_eval(self.function)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 330. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/deprecated/parse.py:55
- **Description:** pickle load/dumps
- **Evidence (Line):** `return pickle.loads(bb)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 331. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/json_schema.py:1836
- **Description:** eval()
- **Evidence (Line):** `if len(_typing_extra.signature_no_eval(json_schema_extra).parameters) > 1:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 332. [Critical] Hardcoded Secret
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/types.py:1835
- **Description:** Password Assignment detected
- **Evidence (Line):** `user = User(username='scolvin', password='password1')`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 333. [Critical] Hardcoded Secret
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/types.py:1862
- **Description:** Password Assignment detected
- **Evidence (Line):** `model = Model(password='IAmSensitive', password_bytes=b'IAmSensitiveBytes')`
- **Exploit Scenario:** Attacker could extract credentials and gain unauthorized access
- **Fix Recommendation:** Move secrets to environment variables or secret manager, rotate compromised credentials
- **Confidence:** High

### 334. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/v1/parse.py:42
- **Description:** pickle load/dumps
- **Evidence (Line):** `return pickle.loads(bb)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 335. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/v1/utils.py:195
- **Description:** eval()
- **Evidence (Line):** `eval('__IPYTHON__')`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 336. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pydantic/validate_call_decorator.py:29
- **Description:** eval()
- **Evidence (Line):** `_typing_extra.signature_no_eval(cast(_generate_schema.ValidateCallSupportedTypes, function))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 337. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/formatters/__init__.py:91
- **Description:** eval()
- **Evidence (Line):** `this method is equivalent to running ``eval()`` on the input file. The formatter is`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 338. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/formatters/__init__.py:103
- **Description:** exec()
- **Evidence (Line):** `exec(f.read(), custom_namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 339. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/lexers/__init__.py:154
- **Description:** exec()
- **Evidence (Line):** `exec(f.read(), custom_namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 340. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/lexers/_julia_builtins.py:150
- **Description:** eval()
- **Evidence (Line):** `v = eval(Symbol(compl.mod))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 341. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/lexers/_julia_builtins.py:361
- **Description:** eval()
- **Evidence (Line):** `v = eval(Symbol(compl.mod))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 342. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/pygments/lexers/special.py:115
- **Description:** eval()
- **Evidence (Line):** `val = ast.literal_eval(val)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 343. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/array_api_compat/_internal.py:64
- **Description:** exec()
- **Evidence (Line):** `exec(f"from {mod.__name__} import *", objs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 344. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/array_api_compat/cupy/fft.py:7
- **Description:** exec()
- **Evidence (Line):** `exec("from cupy.fft import *", _n)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 345. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/array_api_compat/cupy/linalg.py:10
- **Description:** exec()
- **Evidence (Line):** `exec('from cupy.linalg import *', _n)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 346. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/main.py:650
- **Description:** eval()
- **Evidence (Line):** `fun_val, cub_val, ceq_val = _eval(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 347. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/main.py:697
- **Description:** eval()
- **Evidence (Line):** `fun_val, cub_val, ceq_val = _eval(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 348. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/main.py:840
- **Description:** eval()
- **Evidence (Line):** `fun_val, cub_val, ceq_val = _eval(pb, framework, step, options)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 349. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/main.py:1422
- **Description:** eval()
- **Evidence (Line):** `def _eval(pb, framework, step, options):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 350. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/main.py:1446
- **Description:** eval()
- **Evidence (Line):** `x, fun, maxcv = pb.best_eval(penalty)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 351. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/problem.py:80
- **Description:** eval()
- **Evidence (Line):** `def n_eval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 352. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/problem.py:560
- **Description:** eval()
- **Evidence (Line):** `def n_eval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 353. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/problem.py:886
- **Description:** eval()
- **Evidence (Line):** `x_best, fun_best, _ = self.best_eval(penalty)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 354. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/problem.py:947
- **Description:** eval()
- **Evidence (Line):** `def n_eval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 355. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_external/cobyqa/problem.py:1192
- **Description:** eval()
- **Evidence (Line):** `def best_eval(self, penalty):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 356. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_lib/_bunch.py:160
- **Description:** exec()
- **Evidence (Line):** `exec(s, namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 357. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_lib/_elementwise_iterative_method.py:238
- **Description:** eval()
- **Evidence (Line):** `x = pre_func_eval(work)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 358. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_lib/_elementwise_iterative_method.py:261
- **Description:** eval()
- **Evidence (Line):** `post_func_eval(x, f, work)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 359. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/_lib/tests/test_bunch.py:59
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 360. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/datasets/_fetchers.py:79
- **Description:** pickle load/dumps
- **Evidence (Line):** `ascent = array(pickle.load(f))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 361. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/differentiate/_differentiate.py:449
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 362. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/differentiate/_differentiate.py:495
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, f, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 363. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/integrate/_ivp/tests/test_ivp.py:700
- **Description:** eval()
- **Evidence (Line):** `def test_t_eval():`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 364. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/integrate/_tanhsinh.py:399
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 365. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/integrate/_tanhsinh.py:413
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, fj, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 366. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/integrate/tests/test__quad_vec.py:146
- **Description:** eval()
- **Evidence (Line):** `def test_num_eval(self, quadrature):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 367. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_bsplines.py:160
- **Description:** eval()
- **Evidence (Line):** `def test_rndm_naive_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 368. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_bsplines.py:167
- **Description:** eval()
- **Evidence (Line):** `y_n = xp.stack([_naive_eval(x, t, c, k, xp=xp) for x in xx])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 369. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_bsplines.py:941
- **Description:** eval()
- **Evidence (Line):** `def _naive_eval(x, t, c, k, *, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 370. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_bsplines.py:1417
- **Description:** eval()
- **Evidence (Line):** `b1 = np.vectorize(lambda x: _naive_eval(x, t, c, k, xp=np))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 371. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_fitpack2.py:1586
- **Description:** eval()
- **Evidence (Line):** `z_spl = spl_eval(spl, x, y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 372. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_fitpack2.py:1615
- **Description:** eval()
- **Evidence (Line):** `z_spl = spl_eval(spl, x, y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 373. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_fitpack2.py:1651
- **Description:** eval()
- **Evidence (Line):** `z_spl = spl_eval(spl, x, y)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 374. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpnd.py:172
- **Description:** pickle load/dumps
- **Evidence (Line):** `ip2 = pickle.loads(pickle.dumps(ip))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 375. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpnd.py:413
- **Description:** pickle load/dumps
- **Evidence (Line):** `ip2 = pickle.loads(pickle.dumps(ip))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 376. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:858
- **Description:** eval()
- **Evidence (Line):** `def test_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 377. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:1460
- **Description:** eval()
- **Evidence (Line):** `def test_derivative_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 378. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2440
- **Description:** eval()
- **Evidence (Line):** `v2 = _ppoly2d_eval(c, (x, y), xi, yi)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 379. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2446
- **Description:** eval()
- **Evidence (Line):** `v2 = _ppoly2d_eval(c, (x, y), xi, yi, nu=nu)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 380. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2466
- **Description:** eval()
- **Evidence (Line):** `v2 = _ppoly3d_eval(c, (x, y, z), xi, yi, zi, nu=nu)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 381. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2486
- **Description:** eval()
- **Evidence (Line):** `v2 = _ppoly4d_eval(c, (x, y, z, u), xi, yi, zi, ui)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 382. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2689
- **Description:** eval()
- **Evidence (Line):** `def _ppoly2d_eval(c, xs, xnew, ynew, nu=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 383. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2725
- **Description:** eval()
- **Evidence (Line):** `def _ppoly3d_eval(c, xs, xnew, ynew, znew, nu=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 384. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2765
- **Description:** eval()
- **Evidence (Line):** `def _ppoly4d_eval(c, xs, xnew, ynew, znew, unew, nu=None):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 385. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2825
- **Description:** pickle load/dumps
- **Evidence (Line):** `y2 = pickle.loads(pickle.dumps(obj))(x_eval)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 386. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_interpolate.py:2850
- **Description:** pickle load/dumps
- **Evidence (Line):** `Z2 = pickle.loads(pickle.dumps(obj))(eval_points)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 387. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_polyint.py:789
- **Description:** eval()
- **Evidence (Line):** `def test_periodic_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 388. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/interpolate/tests/test_rbfinterp.py:459
- **Description:** pickle load/dumps
- **Evidence (Line):** `yitp2 = pickle.loads(pickle.dumps(interp))(xitp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 389. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/odr/tests/test_odr.py:575
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(obj_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 390. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/odr/tests/test_odr.py:584
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(obj_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 391. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/odr/tests/test_odr.py:588
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(obj_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 392. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/odr/tests/test_odr.py:597
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(obj_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 393. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/odr/tests/test_odr.py:606
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(obj_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 394. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_bracket.py:244
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 395. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_bracket.py:262
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, f, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 396. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_bracket.py:653
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 397. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_bracket.py:669
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, f, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 398. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_chandrupatla.py:152
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 399. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_chandrupatla.py:157
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, f, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 400. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_chandrupatla.py:427
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 401. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_chandrupatla.py:464
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(x, f, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 402. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_nonlin.py:1642
- **Description:** exec()
- **Evidence (Line):** `exec(wrapper, ns)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 403. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/_optimize.py:323
- **Description:** eval()
- **Evidence (Line):** `def eval(x):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 404. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/tests/test__differential_evolution.py:261
- **Description:** eval()
- **Evidence (Line):** `def test_best_solution_retrieval(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 405. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/optimize/tests/test_constraints.py:183
- **Description:** eval()
- **Evidence (Line):** `bounds2 = eval(repr(Bounds(*args)))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 406. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:342
- **Description:** eval()
- **Evidence (Line):** `>>> filtered = cspline1d_eval(cspline1d(sig), time)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 407. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:396
- **Description:** eval()
- **Evidence (Line):** `>>> filtered = qspline1d_eval(qspline1d(sig), time)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 408. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:526
- **Description:** eval()
- **Evidence (Line):** `def cspline1d_eval(cj, newx, dx=1.0, x0=0):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 409. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:569
- **Description:** eval()
- **Evidence (Line):** `>>> filtered = cspline1d_eval(cspline1d(sig), time)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 410. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:592
- **Description:** eval()
- **Evidence (Line):** `res[cond1] = cspline1d_eval(cj, -newx[cond1])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 411. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:593
- **Description:** eval()
- **Evidence (Line):** `res[cond2] = cspline1d_eval(cj, 2 * (N - 1) - newx[cond2])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 412. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:607
- **Description:** eval()
- **Evidence (Line):** `def qspline1d_eval(cj, newx, dx=1.0, x0=0):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 413. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:652
- **Description:** eval()
- **Evidence (Line):** `>>> filtered = qspline1d_eval(qspline1d(sig), time)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 414. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:675
- **Description:** eval()
- **Evidence (Line):** `res[cond1] = qspline1d_eval(cj, -newx[cond1])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 415. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/_spline_filters.py:676
- **Description:** eval()
- **Evidence (Line):** `res[cond2] = qspline1d_eval(cj, 2 * (N - 1) - newx[cond2])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 416. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:164
- **Description:** eval()
- **Evidence (Line):** `def test_cspline1d_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 417. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:165
- **Description:** eval()
- **Evidence (Line):** `r = signal.cspline1d_eval(xp.asarray([0., 0], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 418. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:169
- **Description:** eval()
- **Evidence (Line):** `r = signal.cspline1d_eval(xp.asarray([1., 0, 1], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 419. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:174
- **Description:** eval()
- **Evidence (Line):** `r = signal.cspline1d_eval(xp.asarray([1.0, 0, 1], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 420. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:195
- **Description:** eval()
- **Evidence (Line):** `signal.cspline1d_eval(cj, xp.asarray(newx), dx=dx, x0=x[0]), newy`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 421. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:200
- **Description:** eval()
- **Evidence (Line):** `signal.cspline1d_eval(xp.asarray([], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 422. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:205
- **Description:** eval()
- **Evidence (Line):** `def test_qspline1d_eval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 423. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:206
- **Description:** eval()
- **Evidence (Line):** `xp_assert_close(signal.qspline1d_eval(xp.asarray([0., 0]), xp.asarray([0.])),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 424. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:209
- **Description:** eval()
- **Evidence (Line):** `xp_assert_equal(signal.qspline1d_eval(xp.asarray([1., 0, 1]), xp.asarray([])),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 425. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:214
- **Description:** eval()
- **Evidence (Line):** `r = signal.qspline1d_eval(xp.asarray([1.0, 0, 1], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 426. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:234
- **Description:** eval()
- **Evidence (Line):** `r = signal.qspline1d_eval(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 427. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_bsplines.py:241
- **Description:** eval()
- **Evidence (Line):** `signal.qspline1d_eval(xp.asarray([], dtype=xp.float64),`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 428. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_signaltools.py:1686
- **Description:** eval()
- **Evidence (Line):** `y2 = signal.cspline1d_eval(cj, x2, dx=dx, x0=x[0])`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 429. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/signal/tests/test_signaltools.py:1704
- **Description:** eval()
- **Evidence (Line):** `ynew = signal.cspline1d_eval(cy, xnew)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 430. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/sparse/csgraph/tests/test_graph_laplacian.py:43
- **Description:** eval()
- **Evidence (Line):** `mat = eval(mat, dict(np=np, sparse=sparse))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 431. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/sparse/csgraph/tests/test_graph_laplacian.py:299
- **Description:** eval()
- **Evidence (Line):** `d[L] = eval(L)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 432. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/sparse/csgraph/tests/test_graph_laplacian.py:301
- **Description:** eval()
- **Evidence (Line):** `d[L] = eval(L)(np.eye(n, dtype=mat.dtype))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 433. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/sparse/linalg/tests/test_interface.py:927
- **Description:** pickle load/dumps
- **Evidence (Line):** `B = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 434. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/sparse/tests/test_base.py:2334
- **Description:** pickle load/dumps
- **Evidence (Line):** `sploaded = pickle.loads(pickle.dumps(datsp, protocol=protocol))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 435. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:424
- **Description:** eval()
- **Evidence (Line):** `cdist(X1, X2, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 436. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:430
- **Description:** eval()
- **Evidence (Line):** `cdist(X1, X2, metric=eval(metric), *args)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 437. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:537
- **Description:** eval()
- **Evidence (Line):** `y2 = cdist(X1, X2, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 438. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:547
- **Description:** eval()
- **Evidence (Line):** `cdist(X1, X2, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 439. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:714
- **Description:** eval()
- **Evidence (Line):** `pdist(X1, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 440. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:720
- **Description:** eval()
- **Evidence (Line):** `pdist(X1, metric=eval(metric), *args)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 441. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:1352
- **Description:** eval()
- **Evidence (Line):** `y2 = pdist(X, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 442. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:1362
- **Description:** eval()
- **Evidence (Line):** `pdist(X, metric=eval(metric), **kwargs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 443. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_distance.py:2021
- **Description:** eval()
- **Evidence (Line):** `for m in [metric, eval(metric), "test_" + metric]:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 444. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_kdtree.py:911
- **Description:** pickle load/dumps
- **Evidence (Line):** `T2 = pickle.loads(tmp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 445. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/tests/test_kdtree.py:925
- **Description:** pickle load/dumps
- **Evidence (Line):** `T2 = pickle.loads(tmp)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 446. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/transform/tests/test_rigid_transform.py:1502
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(pkl)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 447. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/spatial/transform/tests/test_rotation.py:2611
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(pkl)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 448. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/special/_precompute/wright_bessel.py:183
- **Description:** eval()
- **Evidence (Line):** `def eval(cls, n, rho, v):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 449. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/special/_precompute/wright_bessel.py:203
- **Description:** eval()
- **Evidence (Line):** `def eval(cls, m, rho, beta):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 450. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/special/_ufunc_tools.py:159
- **Description:** exec()
- **Evidence (Line):** `exec(code, namespace)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 451. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/special/tests/test_orthogonal.py:282
- **Description:** eval()
- **Evidence (Line):** `p = eval(pstr)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 452. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/special/tests/test_support_alternative_backends.py:326
- **Description:** pickle load/dumps
- **Evidence (Line):** `roundtrip = pickle.loads(pickle.dumps(func))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 453. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_continued_fraction.py:325
- **Description:** eval()
- **Evidence (Line):** `def pre_func_eval(work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 454. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_continued_fraction.py:329
- **Description:** eval()
- **Evidence (Line):** `def post_func_eval(n, ab, work):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 455. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_continuous_distns.py:3482
- **Description:** eval()
- **Evidence (Line):** `def sk1_eval(c, *args):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 456. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_continuous_distns.py:3494
- **Description:** eval()
- **Evidence (Line):** `def ku1_eval(c, *args):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 457. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_distn_infrastructure.py:369
- **Description:** exec()
- **Evidence (Line):** `exec('del ' + obj)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 458. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/_distn_infrastructure.py:748
- **Description:** exec()
- **Evidence (Line):** `exec(self._parse_arg_template, ns)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 459. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/common_tests.py:294
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 460. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/common_tests.py:307
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(pkl)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 461. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/common_tests.py:317
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_fit_function = pickle.loads(pickled_fit_function)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 462. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_axis_nan_policy.py:839
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_hypotest = pickle.loads(pickled_hypotest)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 463. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_continued_fraction.py:144
- **Description:** eval()
- **Evidence (Line):** `def test_feval(self, xp):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 464. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_continuous.py:87
- **Description:** eval()
- **Evidence (Line):** `ref = eval(f'(a {left_comparison} x) & (x {right_comparison} b)')`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 465. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_continuous.py:2138
- **Description:** eval()
- **Evidence (Line):** `new_dist = eval(repr(dist))`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 466. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_continuous.py:994
- **Description:** pickle load/dumps
- **Evidence (Line):** `dist3 = pickle.loads(pickle.dumps(dist1))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 467. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_distributions.py:6988
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 468. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_multivariate.py:4396
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 469. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/scipy/stats/tests/test_sampling.py:256
- **Description:** pickle load/dumps
- **Evidence (Line):** `rng2 = pickle.loads(obj)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 470. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/legacy/test_persist.py:15
- **Description:** pickle load/dumps
- **Evidence (Line):** `q = pickle.loads(data)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 471. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/legacy/test_pickle.py:66
- **Description:** pickle load/dumps
- **Evidence (Line):** `result = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 472. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_io.py:1149
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert_geometries_equal(pickle.loads(pickled), geom, tolerance=0)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 473. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_io.py:1155
- **Description:** pickle load/dumps
- **Evidence (Line):** `actual = pickle.loads(pickled)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 474. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_io.py:1170
- **Description:** pickle load/dumps
- **Evidence (Line):** `actual = pickle.loads(pickled)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 475. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_io.py:1184
- **Description:** pickle load/dumps
- **Evidence (Line):** `actual = pickle.loads(pickled)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 476. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_io.py:1196
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert shapely.get_srid(pickle.loads(pickled)) == 4326`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 477. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/shapely/tests/test_strtree.py:133
- **Description:** pickle load/dumps
- **Evidence (Line):** `tree = pickle.loads(pickled_strtree)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 478. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/_loss/tests/test_loss.py:1329
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_loss = pickle.loads(pickled_loss)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 479. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/callback/tests/test_pickle.py:46
- **Description:** pickle load/dumps
- **Evidence (Line):** `restored = pickle.loads(pickle.dumps(estimator))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 480. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/callback/tests/test_pickle.py:57
- **Description:** pickle load/dumps
- **Evidence (Line):** `restored = pickle.loads(pickle.dumps(estimator))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 481. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/callback/tests/test_pickle.py:83
- **Description:** pickle load/dumps
- **Evidence (Line):** `restored = pickle.loads(pickle.dumps(estimator))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 482. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/callback/tests/test_pickle.py:127
- **Description:** pickle load/dumps
- **Evidence (Line):** `est = pickle.load(f)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 483. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/callback/tests/test_scoring_monitor.py:325
- **Description:** eval()
- **Evidence (Line):** `def test_get_logs_include_lineage_ancestor_retrieval(as_pandas):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 484. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/cluster/tests/test_dbscan.py:299
- **Description:** pickle load/dumps
- **Evidence (Line):** `assert type(pickle.loads(s)) is obj.__class__`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 485. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/cluster/tests/test_spectral.py:72
- **Description:** pickle load/dumps
- **Evidence (Line):** `model_copy = pickle.loads(pickle.dumps(model))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 486. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/compose/tests/test_column_transformer.py:1464
- **Description:** pickle load/dumps
- **Evidence (Line):** `selector_picked = pickle.loads(pickle.dumps(selector))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 487. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/datasets/_twenty_newsgroups.py:310
- **Description:** pickle load/dumps
- **Evidence (Line):** `cache = pickle.loads(uncompressed_content)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 488. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/ensemble/_hist_gradient_boosting/tests/test_gradient_boosting.py:1719
- **Description:** pickle load/dumps
- **Evidence (Line):** `new_clf = pickle.load(pickle_dump_with_different_bitness())`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 489. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/ensemble/tests/test_forest.py:721
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj2 = pickle.loads(pickle_object)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 490. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/ensemble/tests/test_gradient_boosting.py:485
- **Description:** pickle load/dumps
- **Evidence (Line):** `clf = pickle.loads(serialized_clf)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 491. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/ensemble/tests/test_weight_boosting.py:189
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj2 = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 492. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/ensemble/tests/test_weight_boosting.py:200
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj2 = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 493. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/externals/array_api_compat/_internal.py:64
- **Description:** exec()
- **Evidence (Line):** `exec(f"from {mod.__name__} import *", objs)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 494. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/externals/array_api_compat/cupy/fft.py:7
- **Description:** exec()
- **Evidence (Line):** `exec("from cupy.fft import *", _n)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 495. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/externals/array_api_compat/cupy/linalg.py:10
- **Description:** exec()
- **Evidence (Line):** `exec('from cupy.linalg import *', _n)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 496. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/feature_extraction/tests/test_text.py:1051
- **Description:** pickle load/dumps
- **Evidence (Line):** `copy = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 497. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/feature_extraction/tests/test_text.py:1075
- **Description:** pickle load/dumps
- **Evidence (Line):** `roundtripped_function = pickle.loads(pickle.dumps(function))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 498. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/feature_extraction/tests/test_text.py:1101
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_cv = pickle.loads(pickle.dumps(cv))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 499. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/feature_extraction/tests/test_text.py:1130
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_cv = pickle.loads(pickle.dumps(cv))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 500. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/feature_extraction/tests/test_text.py:1142
- **Description:** pickle load/dumps
- **Evidence (Line):** `copy = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 501. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/linear_model/tests/test_sgd.py:919
- **Description:** pickle load/dumps
- **Evidence (Line):** `clf = pickle.loads(pickle.dumps(clf))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 502. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/_ranking.py:1749
- **Description:** eval()
- **Evidence (Line):** `European conference on information retrieval (pp. 414-421). Springer,`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 503. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/_ranking.py:1855
- **Description:** eval()
- **Evidence (Line):** `European conference on information retrieval (pp. 414-421). Springer,`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 504. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/_ranking.py:2017
- **Description:** eval()
- **Evidence (Line):** `European conference on information retrieval (pp. 414-421). Springer,`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 505. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/tests/test_dist_metrics.py:258
- **Description:** pickle load/dumps
- **Evidence (Line):** `dm2 = pickle.loads(pickle.dumps(dm))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 506. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/tests/test_dist_metrics.py:268
- **Description:** pickle load/dumps
- **Evidence (Line):** `dm2 = pickle.loads(pickle.dumps(dm))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 507. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/tests/test_dist_metrics.py:327
- **Description:** pickle load/dumps
- **Evidence (Line):** `euclidean_pkl = pickle.loads(pickle.dumps(euclidean))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 508. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/tests/test_dist_metrics.py:328
- **Description:** pickle load/dumps
- **Evidence (Line):** `pyfunc_pkl = pickle.loads(pickle.dumps(pyfunc))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 509. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/metrics/tests/test_score_objects.py:458
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_scorer = pickle.loads(pickle.dumps(scorer))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 510. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/model_selection/tests/test_search.py:1618
- **Description:** pickle load/dumps
- **Evidence (Line):** `grid_search_pickled = pickle.loads(pickle.dumps(grid_search))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 511. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/model_selection/tests/test_search.py:1625
- **Description:** pickle load/dumps
- **Evidence (Line):** `random_search_pickled = pickle.loads(pickle.dumps(random_search))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 512. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/neighbors/tests/test_nearest_centroid.py:116
- **Description:** pickle load/dumps
- **Evidence (Line):** `obj2 = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 513. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/neighbors/tests/test_neighbors_tree.py:289
- **Description:** pickle load/dumps
- **Evidence (Line):** `tree2 = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 514. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/neighbors/tests/test_quad_tree.py:74
- **Description:** pickle load/dumps
- **Evidence (Line):** `bt2 = pickle.loads(s)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 515. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:507
- **Description:** pickle load/dumps
- **Evidence (Line):** `tree_restored = pickle.loads(tree_pickle)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 516. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:539
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(tree_pickle_other)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 517. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:567
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(tree_pickle_noversion)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 518. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:583
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(tree_pickle_noversion)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 519. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:610
- **Description:** pickle load/dumps
- **Evidence (Line):** `estimator_restored = pickle.loads(serialized)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 520. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:651
- **Description:** pickle load/dumps
- **Evidence (Line):** `estimator_restored = pickle.loads(serialized)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 521. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_base.py:891
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(pickle.dumps(BaseEstimator()))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 522. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tests/test_isotonic.py:379
- **Description:** pickle load/dumps
- **Evidence (Line):** `ir2 = pickle.loads(ir_ser)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 523. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tree/tests/test_tree.py:909
- **Description:** pickle load/dumps
- **Evidence (Line):** `est2 = pickle.loads(serialized_object)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 524. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tree/tests/test_tree.py:1849
- **Description:** pickle load/dumps
- **Evidence (Line):** `return pickle.loads(pickle.dumps(obj))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 525. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tree/tests/test_tree.py:2155
- **Description:** pickle load/dumps
- **Evidence (Line):** `new_clf = pickle.load(get_pickle_non_native_endianness())`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 526. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tree/tests/test_tree.py:2249
- **Description:** pickle load/dumps
- **Evidence (Line):** `new_clf = pickle.load(pickle_dump_with_different_bitness())`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 527. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/tree/tests/test_tree.py:2397
- **Description:** pickle load/dumps
- **Evidence (Line):** `splitter_back = pickle.loads(splitter_serialize)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 528. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/estimator_checks.py:2717
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_estimator = pickle.loads(pickled_estimator)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 529. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/estimator_checks.py:3975
- **Description:** pickle load/dumps
- **Evidence (Line):** `est = pickle.loads(pickle.dumps(est))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 530. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_deprecation.py:89
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads(pickle.dumps(mock_function))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 531. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_encode.py:140
- **Description:** pickle load/dumps
- **Evidence (Line):** `uniques = pickle.loads(pickle.dumps(uniques))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 532. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_encode.py:149
- **Description:** pickle load/dumps
- **Evidence (Line):** `uniques = pickle.loads(pickle.dumps(uniques))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 533. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_encode.py:159
- **Description:** pickle load/dumps
- **Evidence (Line):** `uniques = pickle.loads(pickle.dumps(uniques))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 534. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_encode.py:182
- **Description:** pickle load/dumps
- **Evidence (Line):** `uniques = pickle.loads(pickle.dumps(uniques))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 535. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/utils/tests/test_metaestimators.py:62
- **Description:** pickle load/dumps
- **Evidence (Line):** `unpickled_func = pickle.loads(pickled_bytes)`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 536. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mssql/base.py:1857
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 537. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mssql/base.py:1918
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 538. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/_mariadb_shim.py:81
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self) -> None:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 539. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/mariadbconnector.py:103
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self) -> None:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 540. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/mariadbconnector.py:104
- **Description:** exec()
- **Evidence (Line):** `super().post_exec()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 541. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/base.py:2078
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 542. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/cx_oracle.py:1117
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 543. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/cx_oracle.py:1118
- **Description:** exec()
- **Evidence (Line):** `super().pre_exec()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 544. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/cx_oracle.py:1128
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 545. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/asyncpg.py:483
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 546. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/pg8000.py:327
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 547. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/psycopg2.py:568
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 548. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/base.py:1827
- **Description:** exec()
- **Evidence (Line):** `context.pre_exec()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 549. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/base.py:1967
- **Description:** exec()
- **Evidence (Line):** `context.post_exec()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 550. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/base.py:2134
- **Description:** exec()
- **Evidence (Line):** `context.post_exec()`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 551. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/default.py:2042
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 552. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/default.py:2050
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 553. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/default.py:2070
- **Description:** exec()
- **Evidence (Line):** `calculated within post_exec().`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 554. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/interfaces.py:3189
- **Description:** exec()
- **Evidence (Line):** `sql.Compiled object by the time pre_exec() has completed."""`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 555. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/interfaces.py:3333
- **Description:** exec()
- **Evidence (Line):** `def pre_exec(self) -> None:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 556. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/interfaces.py:3379
- **Description:** exec()
- **Evidence (Line):** `def post_exec(self) -> None:`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 557. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/ext/serializer.py:64
- **Description:** pickle load/dumps
- **Evidence (Line):** `pickle.loads()/dumps() can be used to fully dump any ``MetaData`` object,`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 558. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/ext/serializer.py:149
- **Description:** pickle load/dumps
- **Evidence (Line):** `cls = pickle.loads(b64decode(clsarg))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 559. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/ext/serializer.py:152
- **Description:** pickle load/dumps
- **Evidence (Line):** `cls = pickle.loads(b64decode(args))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 560. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/ext/serializer.py:155
- **Description:** pickle load/dumps
- **Evidence (Line):** `cls = pickle.loads(b64decode(args))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 561. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/ext/serializer.py:159
- **Description:** pickle load/dumps
- **Evidence (Line):** `cls = pickle.loads(b64decode(mapper))`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 562. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1323
- **Description:** eval()
- **Evidence (Line):** `registry of tables.  The Python ``eval()`` function is no longer`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 563. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1559
- **Description:** eval()
- **Evidence (Line):** `argument is interpreted using Python's ``eval()`` function.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 564. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1763
- **Description:** eval()
- **Evidence (Line):** `argument is interpreted using Python's ``eval()`` function.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 565. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1860
- **Description:** eval()
- **Evidence (Line):** `argument is interpreted using Python's ``eval()`` function.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 566. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1879
- **Description:** eval()
- **Evidence (Line):** `argument is interpreted using Python's ``eval()`` function.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 567. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/_orm_constructors.py:1921
- **Description:** eval()
- **Evidence (Line):** `argument is interpreted using Python's ``eval()`` function.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 568. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/bulk_persistence.py:694
- **Description:** exec()
- **Evidence (Line):** `def orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 569. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/bulk_persistence.py:1211
- **Description:** exec()
- **Evidence (Line):** `def orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 570. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/clsregistry.py:592
- **Description:** eval()
- **Evidence (Line):** `x = eval(self.arg, globals(), self._dict)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 571. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/context.py:295
- **Description:** exec()
- **Evidence (Line):** `def orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 572. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/context.py:345
- **Description:** exec()
- **Evidence (Line):** `def orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 573. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/context.py:506
- **Description:** exec()
- **Evidence (Line):** `def orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 574. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/events.py:1986
- **Description:** exec()
- **Evidence (Line):** `def after_flush_postexec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 575. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/session.py:2287
- **Description:** exec()
- **Evidence (Line):** `) = compile_state_cls.orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 576. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/session.py:2327
- **Description:** exec()
- **Evidence (Line):** `# now run orm_pre_session_exec() "for real".   if there were`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 577. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/session.py:2336
- **Description:** exec()
- **Evidence (Line):** `) = compile_state_cls.orm_pre_session_exec(`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 578. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/session.py:4674
- **Description:** exec()
- **Evidence (Line):** `self.dispatch.after_flush_postexec(self, flush_context)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 579. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/strategies.py:2923
- **Description:** exec()
- **Evidence (Line):** `def load_collection_from_joined_exec(state, dict_, row):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 580. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/orm/strategies.py:2962
- **Description:** exec()
- **Evidence (Line):** `def load_scalar_from_joined_exec(state, dict_, row):`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 581. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/sql/lambdas.py:1264
- **Description:** exec()
- **Evidence (Line):** `exec(code, vars_, vars_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 582. [Critical] Insecure Deserialization
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/sql/sqltypes.py:2034
- **Description:** pickle load/dumps
- **Evidence (Line):** ```pickle.dumps()`` to incoming objects, and ``pickle.loads()`` on`
- **Exploit Scenario:** Attacker could achieve remote code execution
- **Fix Recommendation:** Use safe serialization formats (JSON), avoid pickle/yaml.load with untrusted data
- **Confidence:** High

### 583. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/testing/plugin/pytestplugin.py:671
- **Description:** exec()
- **Evidence (Line):** `exec(code, env)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 584. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/langhelpers.py:455
- **Description:** exec()
- **Evidence (Line):** `exec(code, env)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 585. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/langhelpers.py:2163
- **Description:** exec()
- **Evidence (Line):** `exec(code, env)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 586. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/typing.py:265
- **Description:** eval()
- **Evidence (Line):** `annotation = eval(expression, cls_namespace, locals_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 587. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/typing.py:267
- **Description:** eval()
- **Evidence (Line):** `annotation = eval(expression, base_globals, locals_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 588. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/typing.py:293
- **Description:** eval()
- **Evidence (Line):** `# name only, just look in globals.  eval() works perfectly fine here,`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 589. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/threadpoolctl.py:1708
- **Description:** exec()
- **Evidence (Line):** `exec(options.command)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 590. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:1591
- **Description:** eval()
- **Evidence (Line):** `(unless you are familiar with how eval() and exec() work).  The`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 591. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:4167
- **Description:** eval()
- **Evidence (Line):** `# as a way of emulating annotation scopes when calling `eval()``
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 592. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:4172
- **Description:** eval()
- **Evidence (Line):** `value if not isinstance(value, str) else eval(value, globals, locals)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 593. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:4219
- **Description:** eval()
- **Evidence (Line):** `# If we pass None to eval() below, the globals of this module are used.`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 594. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:4231
- **Description:** eval()
- **Evidence (Line):** `# as a way of emulating annotation scopes when calling `eval()``
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 595. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:4254
- **Description:** eval()
- **Evidence (Line):** `value = eval(code, globals, locals)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 596. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_extensions.py:1591
- **Description:** exec()
- **Evidence (Line):** `(unless you are familiar with how eval() and exec() work).  The`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 597. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_inspection/typing_objects.py:112
- **Description:** exec()
- **Evidence (Line):** `exec(func_code, globals_, locals_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 598. [Critical] Command Injection
- **File:** .venv-test/lib/python3.12/site-packages/typing_inspection/typing_objects.py:153
- **Description:** exec()
- **Evidence (Line):** `exec(func_code, globals_, locals_)`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 599. [Critical] Command Injection
- **File:** htmlcov/coverage_html_cb_15cffcd0.js:236
- **Description:** exec()
- **Evidence (Line):** `const match = /\.([0-9]+)/.exec(cell.textContent);`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 600. [Critical] Command Injection
- **File:** htmlcov/coverage_html_cb_54cfdca1.js:274
- **Description:** exec()
- **Evidence (Line):** `const match = /\.([0-9]+)/.exec(cell.textContent);`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 601. [Critical] Command Injection
- **File:** htmlcov/coverage_html_cb_dd2e7eb5.js:236
- **Description:** exec()
- **Evidence (Line):** `const match = /\.([0-9]+)/.exec(cell.textContent);`
- **Exploit Scenario:** Attacker could execute arbitrary system commands
- **Fix Recommendation:** Avoid shell execution, use subprocess with shell=False and list args
- **Confidence:** High

### 602. [High] .env File with Secrets
- **File:** .env.example:1
- **Description:** .env.example contains potential secrets
- **Evidence (Line):** `# API Configuration`
- **Exploit Scenario:** Secrets in committed .env files exposed in git history
- **Fix Recommendation:** Add .env to .gitignore, remove from git history, rotate secrets
- **Confidence:** High

### 603. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mssql/base.py:3277
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"SET TRANSACTION ISOLATION LEVEL {level}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 604. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mssql/base.py:2001
- **Description:** SQL % formatting
- **Evidence (Line):** `self.cursor.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 605. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/base.py:2853
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"SET SESSION TRANSACTION ISOLATION LEVEL {level}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 606. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/mysqldb.py:184
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute("SET NAMES %s" % charset_name)`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 607. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/cx_oracle.py:1399
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"ALTER SESSION SET ISOLATION_LEVEL={level}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 608. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/oracle/provision.py:277
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute("ALTER SESSION SET CURRENT_SCHEMA=%s" % schema_name)`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 609. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/base.py:364
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute("SET SESSION search_path='%s'" % schema_name)`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 610. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/pg8000.py:368
- **Description:** SQL string concatenation
- **Evidence (Line):** `self.cursor.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 611. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/pg8000.py:579
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"""`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 612. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/pg8000.py:538
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 613. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/pg8000.py:559
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 614. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/postgresql/provision.py:90
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute("SET SESSION search_path='%s'" % schema_name)`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 615. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/sqlite/base.py:2335
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"PRAGMA read_uncommitted = {isolation_level}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 616. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/sqlite/provision.py:122
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `dbapi_connection.execute(`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 617. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/sqlite/pysqlcipher.py:150
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"pragma key={passphrase}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 618. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/sqlite/pysqlcipher.py:152
- **Description:** SQL f-string interpolation
- **Evidence (Line):** `cursor.execute(f"pragma {prag}={value}")`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 619. [High] SQL Injection
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/engine/base.py:3089
- **Description:** SQL % formatting
- **Evidence (Line):** `cursor.execute("use %s" % shards[shard_id])`
- **Exploit Scenario:** Attacker could manipulate SQL queries to access/modify/delete data
- **Fix Recommendation:** Use parameterized queries/prepared statements
- **Confidence:** Medium

### 620. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/coverage/lcovreport.py:43
- **Description:** MD5 usage
- **Evidence (Line):** `hashed = hashlib.md5(line.encode("utf-8"), usedforsecurity=False).digest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 621. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/coverage/lcovreport.py:43
- **Description:** Weak hash algorithm
- **Evidence (Line):** `hashed = hashlib.md5(line.encode("utf-8"), usedforsecurity=False).digest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 622. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/duckdb/experimental/spark/sql/functions.py:3423
- **Description:** MD5 usage
- **Evidence (Line):** `def md5(col: "ColumnOrName") -> Column:`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 623. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/duckdb/experimental/spark/sql/functions.py:3443
- **Description:** MD5 usage
- **Evidence (Line):** `>>> spark.createDataFrame([("ABC",)], ["a"]).select(md5("a").alias("hash")).collect()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 624. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:309
- **Description:** SHA1 usage
- **Evidence (Line):** `return hashlib.sha1(s).hexdigest()[:16].encode()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 625. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:177
- **Description:** Weak hash algorithm
- **Evidence (Line):** `"MD5": hashlib.md5,`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 626. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:178
- **Description:** Weak hash algorithm
- **Evidence (Line):** `"MD5-SESS": hashlib.md5,`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 627. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:179
- **Description:** Weak hash algorithm
- **Evidence (Line):** `"SHA": hashlib.sha1,`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 628. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:180
- **Description:** Weak hash algorithm
- **Evidence (Line):** `"SHA-SESS": hashlib.sha1,`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 629. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/httpx/_auth.py:309
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.sha1(s).hexdigest()[:16].encode()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 630. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:212
- **Description:** MD5 usage
- **Evidence (Line):** `In [23]: %timeit hashlib.md5(a).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 631. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:215
- **Description:** MD5 usage
- **Evidence (Line):** `In [24]: %timeit hashlib.md5(pickle.dumps(a, protocol=2)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 632. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:218
- **Description:** MD5 usage
- **Evidence (Line):** `In [25]: %timeit hashlib.md5(cPickle.dumps(a, protocol=2)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 633. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:228
- **Description:** MD5 usage
- **Evidence (Line):** `return hashlib.md5(memoryview(x)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 634. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:212
- **Description:** Weak hash algorithm
- **Evidence (Line):** `In [23]: %timeit hashlib.md5(a).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 635. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:215
- **Description:** Weak hash algorithm
- **Evidence (Line):** `In [24]: %timeit hashlib.md5(pickle.dumps(a, protocol=2)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 636. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:218
- **Description:** Weak hash algorithm
- **Evidence (Line):** `In [25]: %timeit hashlib.md5(cPickle.dumps(a, protocol=2)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 637. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/joblib/test/test_hashing.py:228
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.md5(memoryview(x)).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 638. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:179
- **Description:** MD5 usage
- **Evidence (Line):** `return hashlib.md5(x, usedforsecurity=False).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 639. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:187
- **Description:** SHA1 usage
- **Evidence (Line):** `return hashlib.sha1(x, usedforsecurity=False).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 640. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:237
- **Description:** SHA1 usage
- **Evidence (Line):** `cnonce = hashlib.sha1(s, usedforsecurity=False).hexdigest()[:16]`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 641. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:179
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.md5(x, usedforsecurity=False).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 642. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:187
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.sha1(x, usedforsecurity=False).hexdigest()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 643. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/requests/auth.py:237
- **Description:** Weak hash algorithm
- **Evidence (Line):** `cnonce = hashlib.sha1(s, usedforsecurity=False).hexdigest()[:16]`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 644. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/rich/tree.py:18
- **Description:** DES usage
- **Evidence (Line):** `ASCII_GUIDES (GuideType): Guide lines used when Console.ascii_only is True.`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 645. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/pip/_vendor/rich/tree.py:19
- **Description:** DES usage
- **Evidence (Line):** `TREE_GUIDES (List[GuideType, GuideType, GuideType]): Default guide lines.`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 646. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/datasets/_openml.py:548
- **Description:** MD5 usage
- **Evidence (Line):** `md5 = hashlib.md5()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 647. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sklearn/datasets/_openml.py:548
- **Description:** Weak hash algorithm
- **Evidence (Line):** `md5 = hashlib.md5()`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 648. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/dialects/mysql/base.py:497
- **Description:** MD5 usage
- **Evidence (Line):** `table.select(table.c.password == func.md5("plaintext"))`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 649. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/testing/plugin/pytestplugin.py:561
- **Description:** MD5 usage
- **Evidence (Line):** `digest = hashlib.md5(`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 650. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/testing/plugin/pytestplugin.py:561
- **Description:** Weak hash algorithm
- **Evidence (Line):** `digest = hashlib.md5(`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 651. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/compat.py:138
- **Description:** MD5 usage
- **Evidence (Line):** `return hashlib.md5(usedforsecurity=False)`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High

### 652. [Medium] Weak Cryptography
- **File:** .venv-test/lib/python3.12/site-packages/sqlalchemy/util/compat.py:138
- **Description:** Weak hash algorithm
- **Evidence (Line):** `return hashlib.md5(usedforsecurity=False)`
- **Exploit Scenario:** Weak algorithms vulnerable to collision/brute force attacks
- **Fix Recommendation:** Use modern algorithms (SHA-256+, bcrypt/argon2 for passwords, secrets module for tokens)
- **Confidence:** High
