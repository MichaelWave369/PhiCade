import { useState } from "react";
import {
  buildDosHardwareReport, DOS_HARDWARE_CHECKS, emptyDosHardwareSelections,
  type DosHardwareCheckId, type DosHardwareOutcome,
} from "./dosHardwareEvidence";
import "./dosHardwareCheck.css";

/** No external probes or network calls. This is explicitly human self-report. */
export default function DosHardwareCheck() {
  const [results, setResults] = useState(emptyDosHardwareSelections);
  const [report, setReport] = useState("");
  const [copied, setCopied] = useState(false);
  const update = (id: DosHardwareCheckId, outcome: DosHardwareOutcome) => {
    setResults((current) => ({ ...current, [id]: outcome }));
    setReport("");
    setCopied(false);
  };
  const generate = () => {
    setReport(JSON.stringify(buildDosHardwareReport(results, new Date().toISOString()), null, 2));
    setCopied(false);
  };
  const copy = () => {
    if (!report || !navigator.clipboard?.writeText) {
      setCopied(false);
      return;
    }
    void navigator.clipboard.writeText(report).then(() => setCopied(true))
      .catch(() => setCopied(false));
  };
  return (
    <details className="dos-hardware-check">
      <summary>DOS HARDWARE CHECK // MANUAL EVIDENCE REPORT</summary>
      <p>
        This records <strong>your observations</strong>, not an automated hardware
        certification. Use a DOS game you lawfully possess that supports each
        control. Mark untested checks accordingly.
      </p>
      <div className="dos-hardware-check__rows">
        {DOS_HARDWARE_CHECKS.map((check) => (
          <label key={check.id} className="dos-hardware-check__row">
            <span><strong>{check.label}</strong><small>{check.instruction}</small></span>
            <select
              value={results[check.id]}
              onChange={(event) => update(check.id, event.target.value as DosHardwareOutcome)}
              aria-label={check.label + " result"}
            >
              <option value="not-tested">Not tested</option>
              <option value="pass">Observed pass</option>
              <option value="fail">Observed failure</option>
            </select>
          </label>
        ))}
      </div>
      <div className="dos-hardware-check__actions">
        <button type="button" onClick={generate}>GENERATE LOCAL REPORT</button>
        {report && <button type="button" onClick={copy}>COPY JSON</button>}
        {copied && <small role="status">JSON copied locally.</small>}
      </div>
      {report && (
        <textarea
          value={report}
          readOnly
          rows={10}
          aria-label="Copyable manual DOS hardware report JSON"
          onFocus={(event) => event.currentTarget.select()}
        />
      )}
      <small className="dos-hardware-check__footer">
        No passwords, keystrokes, device IDs or game paths are recorded.
        Nothing is automatically uploaded. CI validates code paths, not physical hardware.
      </small>
    </details>
  );
}
