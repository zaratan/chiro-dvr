"use strict";
const VerdictsHits = (() => {
  const points = (text) =>
    text
      .trim()
      .split(/\s+/)
      .map((pair) => pair.split(",").map(Number))
      .filter((point) => point.length === 2 && point.every(Number.isFinite));

  const toSegment = ([x, y], [ax, ay], [bx, by]) => {
    const [dx, dy] = [bx - ax, by - ay];
    const squared = dx * dx + dy * dy;
    const along = squared === 0 ? 0 : Math.max(0, Math.min(1, ((x - ax) * dx + (y - ay) * dy) / squared));
    return Math.hypot(x - (ax + along * dx), y - (ay + along * dy));
  };

  const toLine = (point, line) => {
    if (line.length === 1) return Math.hypot(point[0] - line[0][0], point[1] - line[0][1]);
    let nearest = Infinity;
    for (let index = 1; index < line.length; index += 1) {
      nearest = Math.min(nearest, toSegment(point, line[index - 1], line[index]));
    }
    return nearest;
  };

  const inside = ([x, y], polygon) => {
    let crossed = false;
    for (let index = 0, previous = polygon.length - 1; index < polygon.length; previous = index, index += 1) {
      const [xi, yi] = polygon[index];
      const [xj, yj] = polygon[previous];
      if (yi > y !== yj > y && x < ((xj - xi) * (y - yi)) / (yj - yi) + xi) crossed = !crossed;
    }
    return crossed;
  };

  const toPolygon = (point, polygon) => (inside(point, polygon) ? 0 : toLine(point, [...polygon, polygon[0]]));

  const toCircle = (point, { x, y, r }) => Math.max(0, Math.hypot(point[0] - x, point[1] - y) - r);

  const distance = (point, shape) =>
    Math.min(
      ...shape.lines.filter((line) => line.length > 0).map((line) => toLine(point, line)),
      ...shape.polygons.filter((polygon) => polygon.length > 2).map((polygon) => toPolygon(point, polygon)),
      ...shape.circles.map((circle) => toCircle(point, circle)),
    );

  const reached = (point, shapes, reach) =>
    shapes
      .map((shape) => ({ id: shape.id, distance: distance(point, shape) }))
      .filter((one) => one.distance <= reach)
      .sort((one, other) => one.distance - other.distance || one.id - other.id)
      .map((one) => one.id);

  const choose = (point, shapes, { reach, current }) => {
    const ids = reached(point, shapes, reach);
    if (ids.length === 0) return null;
    const at = ids.indexOf(current);
    return at === -1 ? ids[0] : ids[(at + 1) % ids.length];
  };

  return { points, distance, reached, choose };
})();
