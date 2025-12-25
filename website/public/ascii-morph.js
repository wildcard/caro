/**
 * AsciiMorph
 * @author Tim Holman (https://github.com/tholman/ascii-morph)
 * MIT Licensed
 */

(function() {
  'use strict';

  var element;
  var canvasDimensions = {};
  var renderedData = [];
  var framesToAnimate = [];
  var myTimeout = null;

  /**
   * Utils
   */
  function extend(target, source) {
    for (var key in source) {
      if (!(key in target)) {
        target[key] = source[key];
      }
    }
    return target;
  }

  function repeat(pattern, count) {
    if (count < 1) return '';
    var result = '';
    while (count > 1) {
      if (count & 1) result += pattern;
      count >>= 1, pattern += pattern;
    }
    return result + pattern;
  }

  function replaceAt(string, index, character) {
    return string.substr(0, index) + character + string.substr(index + character.length);
  }

  /**
   * AsciiMorph
   */
  function init(el, canvasSize) {
    element = el;
    canvasDimensions = canvasSize;
  }

  function squareOutData(data) {
    var i;
    var renderDimensions = {
      x: 0,
      y: data.length
    };

    for (i = 0; i < data.length; i++) {
      if (data[i].length > renderDimensions.x) {
        renderDimensions.x = data[i].length;
      }
    }

    for (i = 0; i < data.length; i++) {
      if (data[i].length < renderDimensions.x) {
        data[i] = (data[i] + repeat(' ', renderDimensions.x - data[i].length));
      }
    }

    var paddings = {
      x: Math.floor((canvasDimensions.x - renderDimensions.x) / 2),
      y: Math.floor((canvasDimensions.y - renderDimensions.y) / 2)
    };

    var paddedData = [];
    var verticalPadding = repeat(' ', canvasDimensions.x);

    for (i = 0; i < paddings.y; i++) {
      paddedData.push(verticalPadding);
    }

    for (i = 0; i < data.length; i++) {
      paddedData.push(repeat(' ', paddings.x) + data[i] + repeat(' ', paddings.x));
    }

    for (i = 0; i < paddings.y; i++) {
      paddedData.push(verticalPadding);
    }

    return paddedData;
  }

  function getMorphedFrame(fromFrame, toFrame, interpolationPoint) {
    var interpolatedFrame = [];
    var fromLine;
    var toLine;
    var midPoint;
    var i;

    for (i = 0; i < fromFrame.length; i++) {
      fromLine = fromFrame[i];
      toLine = toFrame[i];

      midPoint = Math.floor(toLine.length * interpolationPoint);

      interpolatedFrame.push(fromLine.substring(0, midPoint) + toLine.substring(midPoint));
    }
    return interpolatedFrame;
  }

  function renderSquareData(data) {
    var ourData = squareOutData(data.slice());
    renderedData = ourData;
    var renderOutput = '';

    for (var i = 0; i < ourData.length; i++) {
      renderOutput += (ourData[i] + '\n');
    }

    element.innerHTML = renderOutput;
  }

  function render(data) {
    clearTimeout(myTimeout);
    renderSquareData(data);
  }

  function morph(data, frameDelay) {
    clearTimeout(myTimeout);
    frameDelay = frameDelay || 50;

    var frames = [];
    var interpolationStep = 0.05;

    for (var i = 0; i <= 1; i += interpolationStep) {
      frames.push(getMorphedFrame(renderedData, squareOutData(data.slice()), i));
    }

    var frameIndex = 0;

    function animateFrames() {
      if (frameIndex < frames.length) {
        renderedData = frames[frameIndex];
        element.innerHTML = frames[frameIndex].join('\n');
        frameIndex++;
        myTimeout = setTimeout(animateFrames, frameDelay);
      }
    }

    animateFrames();
  }

  /**
   * Expose `AsciiMorph`
   */
  var AsciiMorph = function(el, canvasSize) {
    init(el, canvasSize);
  };

  AsciiMorph.render = render;
  AsciiMorph.morph = morph;

  if (typeof define === 'function' && define.amd) {
    define(function() { return AsciiMorph; });
  } else if (typeof module !== 'undefined' && module.exports) {
    module.exports = AsciiMorph;
  } else {
    window.AsciiMorph = AsciiMorph;
  }
})();
