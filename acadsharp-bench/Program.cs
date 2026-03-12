using System.Diagnostics;
using System.Text.Json;
using ACadSharp;
using ACadSharp.IO;

// ---------------------------------------------------------------------------
// ACadSharp Benchmark Runner
// Called from the Rust harness with:
//   acadsharp-bench --dir <bench_output/scale> --iterations <N>
// Outputs JSON timing results to stdout.
// ---------------------------------------------------------------------------

var dir = "";
var iterations = 5;

for (int i = 0; i < args.Length; i++)
{
    switch (args[i])
    {
        case "--dir" when i + 1 < args.Length:
            dir = args[++i];
            break;
        case "--iterations" when i + 1 < args.Length:
            iterations = int.Parse(args[++i]);
            break;
    }
}

if (string.IsNullOrEmpty(dir) || !Directory.Exists(dir))
{
    Console.Error.WriteLine($"Error: directory '{dir}' does not exist.");
    Environment.Exit(1);
}

var rtDir = Path.Combine(dir, "roundtrip");
Directory.CreateDirectory(rtDir);

var results = new Dictionary<string, List<TimingEntry>>();

// -----------------------------------------------------------------------
// PARSE benchmarks (DXF from disk)
// -----------------------------------------------------------------------
var parseResults = new List<TimingEntry>();
var dxfFiles = new[]
{
    "lines_only.dxf", "circles_only.dxf", "arcs_only.dxf",
    "ellipses_only.dxf", "mixed.dxf", "polylines.dxf", "3d_entities.dxf"
};

foreach (var fileName in dxfFiles)
{
    var path = Path.Combine(dir, fileName);
    if (!File.Exists(path)) continue;

    var label = Path.GetFileNameWithoutExtension(fileName);
    var ms = TimeParseDxf(path, iterations);
    parseResults.Add(new TimingEntry(label, ms));
}
results["parse"] = parseResults;

// -----------------------------------------------------------------------
// WRITE benchmarks (DXF to disk)
// -----------------------------------------------------------------------
var writeResults = new List<TimingEntry>();

// Parse a source to get a document, then time writing it
var mixedSrc = Path.Combine(dir, "mixed.dxf");
var linesSrc = Path.Combine(dir, "lines_only.dxf");

if (File.Exists(linesSrc))
{
    var doc = ReadDxf(linesSrc);
    if (doc != null)
    {
        var outPath = Path.Combine(dir, "write_lines_acadsharp.dxf");
        var ms = TimeWriteDxf(doc, outPath, iterations);
        writeResults.Add(new TimingEntry("lines_only", ms));
    }
}

if (File.Exists(mixedSrc))
{
    var doc = ReadDxf(mixedSrc);
    if (doc != null)
    {
        var outPath = Path.Combine(dir, "write_mixed_acadsharp.dxf");
        var ms = TimeWriteDxf(doc, outPath, iterations);
        writeResults.Add(new TimingEntry("mixed", ms));
    }
}
results["write"] = writeResults;

// -----------------------------------------------------------------------
// ROUNDTRIP benchmarks (read from disk, write to disk)
// -----------------------------------------------------------------------
var rtResults = new List<TimingEntry>();

if (File.Exists(mixedSrc))
{
    var rtPath = Path.Combine(rtDir, "rt_mixed_acadsharp_cs.dxf");
    var ms = TimeRoundtripDxf(mixedSrc, rtPath, iterations);
    rtResults.Add(new TimingEntry("mixed_roundtrip", ms));
}
results["roundtrip"] = rtResults;

// -----------------------------------------------------------------------
// BINARY DXF PARSE benchmarks (DXB from disk)
// -----------------------------------------------------------------------
var binaryParseResults = new List<TimingEntry>();
var dxbFiles = new[] { ("binary_mixed", "binary_mixed.dxb"), ("binary_lines", "binary_lines.dxb") };

foreach (var (label, fileName) in dxbFiles)
{
    var path = Path.Combine(dir, fileName);
    if (!File.Exists(path)) continue;

    var ms = TimeParseDxf(path, iterations);
    binaryParseResults.Add(new TimingEntry(label, ms));
}
results["binary_parse"] = binaryParseResults;

// -----------------------------------------------------------------------
// BINARY DXF ROUNDTRIP benchmarks (read DXB, write DXF back)
// -----------------------------------------------------------------------
var binaryRtResults = new List<TimingEntry>();

var binaryMixedSrc = Path.Combine(dir, "binary_mixed.dxb");
if (File.Exists(binaryMixedSrc))
{
    var rtPath = Path.Combine(rtDir, "rt_binary_mixed_acadsharp_cs.dxf");
    var ms = TimeRoundtripDxf(binaryMixedSrc, rtPath, iterations);
    binaryRtResults.Add(new TimingEntry("binary_mixed_roundtrip", ms));
}
results["binary_roundtrip"] = binaryRtResults;

// -----------------------------------------------------------------------
// DWG PARSE benchmarks
// -----------------------------------------------------------------------
var dwgParseResults = new List<TimingEntry>();
var dwgFiles = new[] { ("dwg_mixed", "mixed.dwg"), ("dwg_lines", "lines.dwg") };

foreach (var (label, fileName) in dwgFiles)
{
    var path = Path.Combine(dir, fileName);
    if (!File.Exists(path)) continue;

    var ms = TimeParseDwg(path, iterations);
    dwgParseResults.Add(new TimingEntry(label, ms));
}
results["dwg_parse"] = dwgParseResults;

// -----------------------------------------------------------------------
// DWG WRITE benchmarks
// -----------------------------------------------------------------------
var dwgWriteResults = new List<TimingEntry>();

if (File.Exists(Path.Combine(dir, "mixed.dwg")))
{
    var doc = ReadDwg(Path.Combine(dir, "mixed.dwg"));
    if (doc != null)
    {
        var outPath = Path.Combine(dir, "write_dwg_mixed_acadsharp.dwg");
        var ms = TimeWriteDwg(doc, outPath, iterations);
        dwgWriteResults.Add(new TimingEntry("dwg_mixed", ms));
    }
}

if (File.Exists(Path.Combine(dir, "lines.dwg")))
{
    var doc = ReadDwg(Path.Combine(dir, "lines.dwg"));
    if (doc != null)
    {
        var outPath = Path.Combine(dir, "write_dwg_lines_acadsharp.dwg");
        var ms = TimeWriteDwg(doc, outPath, iterations);
        dwgWriteResults.Add(new TimingEntry("dwg_lines", ms));
    }
}
results["dwg_write"] = dwgWriteResults;

// -----------------------------------------------------------------------
// DWG ROUNDTRIP benchmarks
// -----------------------------------------------------------------------
var dwgRtResults = new List<TimingEntry>();

if (File.Exists(Path.Combine(dir, "mixed.dwg")))
{
    var rtPath = Path.Combine(rtDir, "rt_mixed_acadsharp_cs.dwg");
    var ms = TimeRoundtripDwg(Path.Combine(dir, "mixed.dwg"), rtPath, iterations);
    dwgRtResults.Add(new TimingEntry("dwg_mixed_roundtrip", ms));
}
results["dwg_roundtrip"] = dwgRtResults;

// -----------------------------------------------------------------------
// Output JSON
// -----------------------------------------------------------------------
var json = JsonSerializer.Serialize(results, new JsonSerializerOptions { WriteIndented = false });
Console.WriteLine(json);

// ===========================================================================
// Helpers
// ===========================================================================

static CadDocument? ReadDxf(string path)
{
    try
    {
        using var reader = new DxfReader(path);
        return reader.Read();
    }
    catch (Exception ex)
    {
        Console.Error.WriteLine($"Warning: failed to read DXF '{path}': {ex.Message}");
        return null;
    }
}

static CadDocument? ReadDwg(string path)
{
    try
    {
        using var reader = new DwgReader(path);
        return reader.Read();
    }
    catch (Exception ex)
    {
        Console.Error.WriteLine($"Warning: failed to read DWG '{path}': {ex.Message}");
        return null;
    }
}

static double TimeParseDxf(string path, int iterations)
{
    // Warmup
    try
    {
        using var r = new DxfReader(path);
        r.Read();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var reader = new DxfReader(path);
            reader.Read();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

static double TimeParseDwg(string path, int iterations)
{
    // Warmup
    try
    {
        using var r = new DwgReader(path);
        r.Read();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var reader = new DwgReader(path);
            reader.Read();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

static double TimeWriteDxf(CadDocument doc, string outPath, int iterations)
{
    // Warmup
    try
    {
        using var w = new DxfWriter(outPath, doc, false);
        w.Write();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var writer = new DxfWriter(outPath, doc, false);
            writer.Write();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

static double TimeWriteDwg(CadDocument doc, string outPath, int iterations)
{
    // Warmup
    try
    {
        using var w = new DwgWriter(outPath, doc);
        w.Write();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var writer = new DwgWriter(outPath, doc);
            writer.Write();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

static double TimeRoundtripDxf(string inputPath, string outputPath, int iterations)
{
    // Warmup
    try
    {
        using var r = new DxfReader(inputPath);
        var doc = r.Read();
        using var w = new DxfWriter(outputPath, doc, false);
        w.Write();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var reader = new DxfReader(inputPath);
            var doc = reader.Read();
            using var writer = new DxfWriter(outputPath, doc, false);
            writer.Write();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

static double TimeRoundtripDwg(string inputPath, string outputPath, int iterations)
{
    // Warmup
    try
    {
        using var r = new DwgReader(inputPath);
        var doc = r.Read();
        using var w = new DwgWriter(outputPath, doc);
        w.Write();
    }
    catch { return double.NaN; }

    var sw = Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        try
        {
            using var reader = new DwgReader(inputPath);
            var doc = reader.Read();
            using var writer = new DwgWriter(outputPath, doc);
            writer.Write();
        }
        catch { return double.NaN; }
    }
    sw.Stop();
    return sw.Elapsed.TotalMilliseconds / iterations;
}

record TimingEntry(string Label, double Ms);
