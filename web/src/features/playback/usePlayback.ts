import { useEffect, useMemo, useRef, useState } from 'react';
import type { LiveProgramDeclarations, ScenarioRequest } from '../../domain/contracts/scenario';
import type { ScenarioReport } from '../../domain/contracts/report';
import { createReplay } from '../../domain/replay/model';
export function usePlayback(
  request: ScenarioRequest,
  report: ScenarioReport,
  visible: boolean,
  liveProgram?: LiveProgramDeclarations,
) {
  const model = useMemo(
    () => createReplay(request, report, liveProgram),
    [request, report, liveProgram],
  );
  const [cursor, setCursor] = useState({ time: 0, limit: Infinity }),
    [playing, setPlaying] = useState(false),
    [speed, setSpeed] = useState(1);
  const timeRef = useRef(0);
  useEffect(() => {
    setPlaying(false);
    timeRef.current = 0;
    setCursor({ time: 0, limit: Infinity });
  }, [model]);
  useEffect(() => {
    if (!visible) setPlaying(false);
  }, [visible]);
  useEffect(() => {
    const hide = () => {
      if (document.hidden) setPlaying(false);
    };
    document.addEventListener('visibilitychange', hide);
    return () => document.removeEventListener('visibilitychange', hide);
  }, []);
  useEffect(() => {
    if (!playing) return;
    let frame: number,
      last: number | null = null;
    const tick = (stamp: number) => {
      const delta = last === null ? 0 : stamp - last;
      last = stamp;
      timeRef.current = Math.min(model.horizon, timeRef.current + delta * 1000 * speed);
      setCursor({ time: timeRef.current, limit: Infinity });
      if (timeRef.current >= model.horizon) {
        setPlaying(false);
        return;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [playing, speed, model]);
  const seek = (time: number, limit = Infinity) => {
    setPlaying(false);
    timeRef.current = Math.max(0, Math.min(model.horizon, time));
    setCursor({ time: timeRef.current, limit });
  };
  const state = useMemo(() => model.stateAt(cursor.time, cursor.limit), [model, cursor]);
  const step = (direction: number) => {
    const count = Math.max(0, Math.min(model.events.length, state.count + direction));
    seek(count ? model.events[count - 1].time : 0, count);
  };
  const toggle = () => {
    if (!playing && cursor.time >= model.horizon) {
      timeRef.current = 0;
      setCursor({ time: 0, limit: Infinity });
    }
    setPlaying((v) => !v);
  };
  return { model, state, playing, speed, setSpeed, seek, step, toggle };
}
export type Playback = ReturnType<typeof usePlayback>;
